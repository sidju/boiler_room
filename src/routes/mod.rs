use hyper::header::HeaderValue;
use hyper::{Method, StatusCode};
use askama::Template;

use crate::{
  State,
  Error,
  ClientError,
  Request,
  Response,
};

// A utils file for common operations while routing
mod utils;
use utils::*;
mod auth;
use auth::*;

// And the actual route modules
mod secure;

#[derive(Template)]
#[template(path = "index.html")]
struct Index{
}

pub async fn route(
  state: &'static State,
  req: Request,
) -> Result<Response, Error> {
  // Put path into a list so we can match on it step by step
  let mut path_vec: Vec<String> = req
    .uri()
    .path()
    .split('/')
    .rev() // Reverse the iterator
    .map(|s| s.to_owned()) // Take ownership of the string, probably clones data
    .collect() // Aggregate into the variable
  ;
  // If the first path is something the uri is malformed
  // (such as http://localhost:8080wrong/path)
  match path_vec.pop().as_deref() {
    None | Some("") => (),
    Some(unexpected) => {
      return Err(Error::path_data_before_root(unexpected.to_owned()));
    },
  }

  // The actual routing
  match path_vec.pop().as_deref() {
    None | Some("") | Some("index.html") => {
      // verify that the path ends here and that the method is correct
      // utility function for simple paths
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Utility function to build html response around given str
      html(Index{}.render()?)
    },
    Some("post-login") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      add_header(
        finish_oidc_login_flow(state, req).await,
        hyper::header::CACHE_CONTROL,
        hyper::header::HeaderValue::from_static("no-store")
      )
    },
    Some("secure") => {
      secure::route(state, req, path_vec).await
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
