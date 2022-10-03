use hyper::header::HeaderValue;
use hyper::{Method, StatusCode};

use crate::{
  State,
  Error,
  Reply,
  Request,
  Response,
};

// A utils file for common operations while routing
mod utils;
use utils::*;

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
      html("Hello world!")
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
