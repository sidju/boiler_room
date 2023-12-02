pub type Response = hyper::Response<http_body_util::Full<hyper::body::Bytes>>;
pub type Request = hyper::Request<hyper::body::Incoming>;

pub trait Reply {
  fn into_response(self) -> Response;
}

impl Reply for Response {
  fn into_response(self) -> Response {
    self
  }
}

impl<T, E> Reply for Result<T, E>
where
  T: Reply,
  E: Reply,
{
  fn into_response(self) -> Response {
    match self {
      Ok(re) => re.into_response(),
      Err(e) => e.into_response(),
    }
  }
}
