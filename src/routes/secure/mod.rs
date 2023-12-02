use super::*;

#[derive(Template)]
#[template(path = "secure/index.html")]
struct Index {
  email: String,
}

pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  // Get out the cookies
  // (For a backend using cookies more than this one, hand in the cookies var to
  // the handlers to provide them access (and perhaps use the `cookie` crate to
  // parse it instead of doing it manually)
  let cookies = parse_cookies(&req)?;

  // Check for session
  let session = match cookies.get("session") {
    Some(id) => {
      // Verify that the id we got is a valid session
      match sqlx::query_as!(SessionData,
        "SELECT session_id, user_id, email
           FROM Sessions
           JOIN Users ON Users.id = Sessions.user_id
         WHERE session_id = $1",
         id,
      )
        .fetch_optional(&state.db)
        .await?
      {
        Some(x) => x,
        None => { return start_oidc_login_flow(state).await; },
      }
    }
    None => {
      return start_oidc_login_flow(state).await;
    }
  };

  println!("{cookies:?}");
  println!("{session:?}");

  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      html(Index{email: session.email}.render()?)
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
