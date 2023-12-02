use super::*;

use serde::{Serialize, Deserialize};
use askama::Template;

#[derive(Template)]
#[template(path = "login_redirect.html")]
struct LoginRedirect<'a>{
  url: &'a str,
}

pub async fn start_oidc_login_flow(
  state: &'static State,
) -> Result<Response, Error> {
  // Create an authorization URL for this user
  let (authorize_url, csrf_state, nonce) = state.oidc_client
    .authorize_url(
      openidconnect::AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
      openidconnect::CsrfToken::new_random,
      openidconnect::Nonce::new_random,
    )
    .add_scope(openidconnect::Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
    .url()
  ;
  // Save all the data we need to keep through the OIDC login process to DB
  sqlx::query!(
    "INSERT INTO LoginProcesses(state_id, nonce) VALUES($1, $2)",
    csrf_state.secret(),
    nonce.secret(),
  )
    .execute(&state.db)
    .await
  ?;

  // Redirect the user to that url
  add_header(
    html(LoginRedirect{url: authorize_url.as_str()}.render()?),
    hyper::header::CACHE_CONTROL,
    hyper::header::HeaderValue::from_static("no-store"),
  )
}

#[derive(Deserialize, Serialize)]
struct PostLoginQueryData{
  code: String,
  state: String,
}
#[derive(sqlx::FromRow, Debug)]
pub struct SessionData {
  pub session_id: String,
  pub user_id: i64,
  pub email: String,
}

pub async fn finish_oidc_login_flow(
  state: &'static State,
  req: Request,
) -> Result<Response, Error> {
  use openidconnect::TokenResponse;

  // Parse out "code" and "state" parameters
  let oidc_response: PostLoginQueryData = parse_query(&req)?;
  // Get all the data about this login from the database, also deleting it
  let nonce = sqlx::query!(
    "SELECT nonce from LoginProcesses WHERE state_id = $1",
    oidc_response.state,
  )
    .fetch_optional(&state.db)
    // Open DB query result to get to Option
    .await?
    // Make Some into Ok and None into this error.
    .ok_or(ClientError::UnknownOIDCProcess)?
    // And get the only row out
    .nonce
  ;

  // If the state was valid we have validated againts CSRF and can request the
  // real code from our OIDC provider
  let code = openidconnect::AuthorizationCode::new(oidc_response.code);
  let token_response = state.oidc_client
    .exchange_code(code)
    .request_async(openidconnect::reqwest::async_http_client)
    .await
  ?;

  // Extract the returned ID token from the response
  let id_token = token_response
    .id_token()
    .ok_or(ClientError::OIDCGaveNoToken)?
  ;
  // Verify this response using the nonce from the DB
  let id_token_verifier = state.oidc_client.id_token_verifier();
  let nonce = openidconnect::Nonce::new(nonce);
  let id_token_claims = id_token
    // And from the token we get the parts we care about like this
    // This means we cryptographically verify it before each use
    .claims(&id_token_verifier, &nonce)? // Errors if auth chain has been tampered with
  ;

  // Now that we have all we need, verify that the user is registered (and get
  // their user id for session creation).
  let email = id_token_claims.email()
    .ok_or(ClientError::OIDCGaveNoEmail)?
    .as_str()
  ;
  let user_id = sqlx::query!(
    "SELECT id FROM Users WHERE email = $1",
    email,
  )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ClientError::UserNotFound(email.to_owned()))?
    .id
  ;

  // At this stage we have the user metadata in id_token_claims and have
  // confirmed the user's identity, so we create a session for them.
  let session_id = nanoid::nanoid!(32);
  sqlx::query(
    "INSERT INTO Sessions(session_id, user_id) VALUES($1, $2)"
  )
    .bind(&session_id)
    .bind(user_id)
    .execute(&state.db)
    .await
  ?;

  // Finally, the horror!
  // We return a website with the session cookie, which has a javascript tag to
  // go back 1 step (aka. to the request that initially triggered the login
  // redirect to the OIDC provider).
  html(
"<!DOCTYPE html>
<html>
  <head>
    <title>Successful login, redirecting you back</title>
    <script type=\"text/javascript\">
      history.back();
    </script>
  </head>
  <body>
    If your browser didn't redirect you, you can get back to where you were
    going by pressing the back button.
  </body>
</html>
"
  )
    .and_then(|mut r| -> Result<Response, Error> {
      r.headers_mut().insert(
        hyper::header::SET_COOKIE,
        HeaderValue::try_from(format!(
          "session={}; Secure; HttpOnly; SameSite=Strict",
          session_id,
        ))?
      );
      Ok(r)
    })

  // Might be nice to revoke the token as well
}
