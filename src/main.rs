use hyper::service::{
  service_fn,
};
use std::net::SocketAddr;
use std::convert::Infallible;

// Define and construct application state (config / data shared across threads)
mod state;
use state::*;

// These define how to convert between errors and responses
// Slightly magicky, but well defined by the std::convert::From/Into traits
mod traits;
use traits::*;
mod error;
use error::*;

// Define how to handle the actual requests
mod routes;

// Wraps the main function in an async runtime
#[tokio::main]
async fn main() {
  let state = init_state().await;
  let addr = SocketAddr::from(([0,0,0,0], 8888));
  run_server(state, addr).await
}

async fn run_server(
  state: &'static State,
  addr: SocketAddr,
) {
  // Listen on address
  let listener = tokio::net::TcpListener::bind(addr).await
    .expect("Failed to bind to socket")
  ;

  // Create whatever background tasks are needed
  tokio::task::spawn(database_cleaner(state));

  // Loop forever, spawning a task for every request we get
  loop {
    let (socket, client_address) = listener.accept().await
      .expect("Failed receive connection")
    ;

    // adapt the stream into something hyper can use
    let io = hyper_util::rt::TokioIo::new(socket);
    // And spawn a task to respond
    tokio::task::spawn(async move {
      // A state machine to:
      // - Parse incoming HTTP request
      // - Hand it into the given service_fn
      // - Serialize the return from function into a HTTP response
      if let Err(e) = hyper::server::conn::http1::Builder::new()
        .serve_connection(io, service_fn(move |req| handle_request(state,req)))
        .await
      {
        eprintln!("Error {e} when serving {client_address}!");
      }
    });
  }
}

// Cleans outdated temporary state from the database every 6 hours
async fn database_cleaner(
  state: &'static State,
) {
  // Do cleanup every six hours
  let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60 * 6));
  loop {
    interval.tick().await;
    // Sessions
    match sqlx::query("DELETE FROM Sessions WHERE valid_until < NOW()")
      .execute(&state.db)
      .await
    {
      Ok(r) => { println!("Cleaned {} outdated sessions.", r.rows_affected()); },
      Err(e) => { eprintln!("Error when cleaning outdated sessions\n  error: {e}"); },
    }
    // Login processes older than 5 minutes are invalid anyways
    match sqlx::query("DELETE FROM LoginProcesses WHERE creation_time < NOW() - INTERVAL '5 minutes'")
      .execute(&state.db)
      .await
    {
      Ok(r) => { println!("Cleaned {} old login processes", r.rows_affected()); },
      Err(e) => { eprintln!("Error when cleaning outdated login procedures\n  error: {e}"); },
    }
  }
}

// The service_fn type requires we hand out an error, but we declare one that
// cannot exist to show that we will never return an error from here
async fn handle_request(
  state: &'static State,
  req: Request,
) -> Result<Response, Infallible> {
  // Call the routing, convert both error and success into a response to return
  Ok(routes::route(state, req).await.into_response())
}
