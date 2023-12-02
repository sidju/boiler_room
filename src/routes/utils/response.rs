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
pub fn html<B: Into<hyper::body::Bytes>>(
  data: B,
) -> Result<Response, Error> {
  let mut re = Response::new(http_body_util::Full::new(data.into()));
  re.headers_mut().insert(
    "Content-Type",
    HeaderValue::from_static("text/html; charset=utf-8")
  );
  Ok(re)
}
// Redirect user to given url
pub fn redirect(
  target: &str,
) -> Result<Response, Error> {
  let mut re = Response::new("".into());
  // Explicitly requires client to GET the given URL
  *re.status_mut() = StatusCode::SEE_OTHER;
  re.headers_mut().insert(
    "Location",
    HeaderValue::from_str(target)?,
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
pub fn add_header(
  re: Result<Response, Error>,
  header: hyper::header::HeaderName,
  value: hyper::header::HeaderValue,
) -> Result<Response, Error> {
  re.map(|mut r| {
    r.headers_mut().append(header, value);
    r
  })
}
