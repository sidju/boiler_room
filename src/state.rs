use std::env::var;
use mongodb::Database;

pub struct State {
  db: Database,
  app_name: String,

  // Only relevant if accepting POST/PUT
  max_content_len: usize,
}

pub async fn init_state() -> &'static State {
  // Read in .env file via dotenv
  dotenv::dotenv().expect("Failed to read .env file into environment");

  // Get needed data from environment
  let max_content_len = var("MAX_CONTENT_LEN")
    .expect("MAX_CONTENT_LEN must be present in environment or .env file")
    .parse::<usize>()
    .expect("MAX_CONTENT_LEN could not be parsed as an unsigned integer")
  ;
  let db_url = var("MONGODB_URL")
    .expect("MONGODB_URL must be present in environment or .env file")
  ;
  let db_name = var("MONGODB_DB")
    .expect("MONGODB_DB must be present in environment or .env file")
  ;
  let app_name = var("APP_NAME")
    .expect("APP_NAME must be present in environment or .env file")
  ;

  // Construct requisite objects
  let mut db_opts = mongodb::options::ClientOptions::parse(db_url)
    .await
    .expect("MONGODB_URL must be a valid connection string")
  ;
  db_opts.app_name = Some(app_name.clone());
  let db_client = mongodb::Client::with_options(db_opts)
    .expect("Failed to connect to mongodb")
  ;
  let db = db_client.database(&db_name);

  // Perform any setup operations

  // Construct and return pointer to eternal instance
  Box::leak(Box::new(State{
    db,
    app_name,
    max_content_len,
  }))
}
