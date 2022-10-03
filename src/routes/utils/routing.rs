use super::*;

//
// Routing helpers
//

// Returns not_found error if there is more path left
pub fn verify_path_end(
  path_vec: &Vec<String>,
  req: &Request,
) -> Result<(), Error> {
  if !path_vec.is_empty() {
    Err(Error::path_not_found(req))
  } else {
    Ok(())
  }
}
// Returns method_not_found error if the method isn't the one given
pub fn verify_method(
  req: &Request,
  expected_method: &Method,
) -> Result<(), Error> {
  if req.method() != expected_method {
    Err(Error::method_not_found(req))
  } else {
    Ok(())
  }
}
// Combines the two above
pub fn verify_method_path_end(
  path_vec: &Vec<String>,
  req: &Request,
  expected_method: &Method,
) -> Result<(), Error> {
  verify_path_end(path_vec, req)?;
  verify_method(req, expected_method)?;
  Ok(())
}
