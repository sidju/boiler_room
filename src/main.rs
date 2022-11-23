use hyper::service::{
  make_service_fn,
  service_fn,
};
use hyper::Server;
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
  let addr = SocketAddr::from(([0,0,0,0], 8080));
  run_server(state, addr).await
}

async fn run_server(
  state: &'static State,
  addr: SocketAddr,
) {
  // Define what to do with requests
  // - A Service is a stateful worker that responds to one request at a time.
  //   service_fn creates a Service from a FnMut that accepts Request and 
  //   returns a Response Future.
  // - A "MakeService" is a Service that creates more Services.
  //   make_service_fn is essentially the same as service_fn, but requiring
  //   that Fn::Return is a Service
  //   Since we can create that from a closure, we can bind in variables to
  //   all created Services
  // - The Services defined match the expected APIs for Tower, so you can use
  //   their pre-routing filters and middlewares. For example you can wrap
  //   the inner service_fn output using tower::Layer::layer, or combine
  //   layers onto a service using tower::builder::ServiceBuilder.
  let make_service = make_service_fn(|_conn| async move {
    Ok::<_, Infallible>(service_fn(move |req| handle_request(state, req)))
  });

  // Create and configure the server
  let server = Server::bind(&addr).serve(make_service);

  // Finally run it all (forever, probably won't ever return)
  match server.await {
    Ok(_) => println!("Server ran successfully"),
    Err(e) => eprintln!("Error occured: {}", e),
  }
}

// The service_fn type requires we hand out an error, but we declare one that
// cannot exist to show that we will never return an error from here
pub async fn handle_request(
  state: &'static State,
  req: Request,
) -> Result<Response, Infallible> {
  // Call the routing, convert both error and success into a response to return
  Ok(routes::route(state, req).await.into_response())
}
