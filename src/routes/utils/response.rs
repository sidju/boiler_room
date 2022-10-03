use super::*;

//
// Response constructors
//

// Return an empty OK response (HTTP 204)
pub fn empty() -> Result<Response, Error> {
  let mut re = Response::new("".into());
  *re.status_mut() = StatusCode::NO_CONTENT;
  Ok(re)
}
// Return an empty not modified response
// Useful to handle http conditional requests
pub fn not_modified() -> Result<Response, Error> {
  let mut re = Response::new("".into());
  *re.status_mut() = StatusCode::NOT_MODIFIED;
  Ok(re)
}
// Return given string as html
pub fn html(
  data: &'static str,
) -> Result<Response, Error> {
  let mut re = Response::new(data.into());
  re.headers_mut().insert(
    "Content-Type",
    HeaderValue::from_static("text/html; charset=utf-8")
  );
  Ok(re)
}
// Return given string as css
pub fn css(
  data: &'static str,
) -> Result<Response, Error> {
  let mut re = Response::new(data.into());
  re.headers_mut().insert(
    "Content-Type",
    HeaderValue::from_static("text/css; charset=utf-8")
  );
  Ok(re)
}
// Serialize given struct into json and return it
pub fn json<T: Serialize + ?Sized>(
  data: &T,
) -> Result<Response, Error> {
  let mut re = Response::new(serde_json::to_string(data)?.into());
  re.headers_mut().insert(
    "Content-Type",
    HeaderValue::from_static("application/json; charset=utf-8"),
  );
  Ok(re)
}

pub fn set_status(
  re: Result<Response, Error>,
  status: StatusCode,
) -> Result<Response, Error> {
  re.map(|mut r| {
    *r.status_mut() = status;
    r
  })
}
