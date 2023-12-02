use std::env::var;
use sqlx::postgres::PgPool;
use serde::{Serialize, Deserialize};


// Teach openidconnect-rs about a Google custom extension to the OpenID Discovery response that we can use as the RFC
// 7009 OAuth 2.0 Token Revocation endpoint. For more information about the Google specific Discovery response see the
// Google OpenID Connect service documentation at: https://developers.google.com/identity/protocols/oidc2/openid-connect#discovery
#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl openidconnect::AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
type GoogleProviderMetadata = openidconnect::ProviderMetadata<
    RevocationEndpointProviderMetadata,
    openidconnect::core::CoreAuthDisplay,
    openidconnect::core::CoreClientAuthMethod,
    openidconnect::core::CoreClaimName,
    openidconnect::core::CoreClaimType,
    openidconnect::core::CoreGrantType,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJweKeyManagementAlgorithm,
    openidconnect::core::CoreJwsSigningAlgorithm,
    openidconnect::core::CoreJsonWebKeyType,
    openidconnect::core::CoreJsonWebKeyUse,
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreResponseMode,
    openidconnect::core::CoreResponseType,
    openidconnect::core::CoreSubjectIdentifierType,
>;

pub struct State {
  pub db: PgPool,
  pub oidc_client: openidconnect::core::CoreClient,

  // Only relevant if accepting POST/PUT
  pub max_content_len: usize,
}

pub async fn init_state() -> &'static State {
  // Read in .env file via dotenv
  dotenvy::dotenv().expect("Failed to read .env file into environment");

  // Get needed data from environment
  let max_content_len = var("MAX_CONTENT_LEN")
    .expect("MAX_CONTENT_LEN must be present in environment or .env file")
    .parse::<usize>()
    .expect("MAX_CONTENT_LEN could not be parsed as an unsigned integer")
  ;
  let db_url = var("DATABASE_URL")
    .expect("DATABASE_URL must be present in environment or .env file")
  ;
  let oidc_client_id = var("OIDC_CLIENT_ID")
    .expect("OIDC_CLIENT_ID must be present in environment or .env file")
  ;
  let oidc_client_secret = var("OIDC_CLIENT_SECRET")
    .expect("OIDC_CLIENT_SECRET must be present in environment or .env file")
  ;
  let mut oidc_redirect_url = var("OIDC_REDIRECT_URI")
    .expect("OIDC_REDIRECT_URI must be present in environment or .env file")
  ;
  oidc_redirect_url.push_str("/post-login");

  // Construct requisite objects
  let db = sqlx::postgres::PgPoolOptions::new()
    .max_connections(8)
    .min_connections(1)
    // It is recommended to enforce reconnects every 24 hours, in case of per
    // connection memory leaks in the database itself
    .max_lifetime(std::time::Duration::from_secs(24 * 60 * 60))
    .connect(&db_url)
    .await
    .expect("Failed to connect to database")
  ;
  let oidc_metadata = GoogleProviderMetadata::discover_async(
    openidconnect::IssuerUrl::new(
      "https://accounts.google.com".to_string()
    ).unwrap(),
    openidconnect::reqwest::async_http_client,
  )
    .await
    .expect("Failed to get oidc metadata from google")
  ;
  let revocation_url = oidc_metadata
    .additional_metadata()
    .revocation_endpoint
    .clone()
  ;
  let oidc_client = openidconnect::core::CoreClient::from_provider_metadata(
    oidc_metadata,
    openidconnect::ClientId::new(oidc_client_id),
    Some(openidconnect::ClientSecret::new(oidc_client_secret)),
  )
    .set_redirect_uri(
      openidconnect::RedirectUrl::new(oidc_redirect_url)
        .expect("Invalid OIDC_REDIRECT_URL")
    )
    .set_revocation_uri(
      openidconnect::RevocationUrl::new(revocation_url)
        .unwrap()
    )
  ;

  // Perform any setup operations
  sqlx::migrate!()
    .run(&db)
    .await
    .expect("Failed to run database migrations. Usually caused by an already applied migration having changed in the source code")
  ;

  // Construct and return pointer to eternal instance
  Box::leak(Box::new(State{
    db,
    oidc_client,
    max_content_len,
  }))
}
