use super::*;

//
// Request deconstructors
//

// Get a specific header as string reference if exists, else None
pub fn get_header<'a>(
  req: &'a Request,
  header_name: &str,
) -> Result<Option<&'a str>, Error> {
  Ok(match req.headers().get(header_name) {
    Some(val) => Some(
      val
        .to_str()
        .map_err(|e| Error::unreadable_header(e, header_name))?,
    ),
    None => None,
  })
}
// Verify content len, should be done before calling get_body or any of the
// other functions below that accept the rest of a multipart request
pub fn validate_get_content_len<'a>(
  req: &'a Request,
  max_len: usize,
) -> Result<usize, Error> {
  let header = get_header(&req, "Content-Length")?;
  if let Some(x) = header {
    let length = x.parse::<usize>().map_err(Error::content_length_not_int)?;
    if length <= max_len {
      Ok(length)
    } else {
      Err(Error::content_length_too_large(length, max_len))
    }
  } else {
    Err(Error::content_length_missing())
  }
}
// Wait for and save packets from client until transmission ends _OR_
// more bytes that Content-Length have been received (error).
// Uses validate_get_content_len to verify Content-Length < max_len
// (
//  More performant than stream processing because Serde performs better
//  on continuous memory, such as a list of bytes.
// )
pub async fn get_body(
  req: &mut Request,
  max_len: usize,
) -> Result<Vec<u8>, Error> {
  use hyper::body::HttpBody;

  // First we validate and set up
  let expected_len = validate_get_content_len(req, max_len)?;
  let mut bytes = Vec::with_capacity(expected_len);
  let body = req.body_mut();
  futures::pin_mut!(body);

  // Then we loop until we either overshoot Content-Len and error or
  // run out of data and return what we got
  while let Some(result) = body.data().await {
    let data = result?;
    // Check against overrunning
    if bytes.len() + data.len() > expected_len {
      // If we overrun try to estimate length of received request
      let estimate = bytes.len() + data.len() + body.size_hint().lower() as usize;
      return Err(Error::content_length_mismatch(estimate, expected_len));
    }

    bytes.extend_from_slice(&data);
  }

  // Finally check against undershooting
  if bytes.len() < expected_len {
    Err(Error::content_length_mismatch(bytes.len(), expected_len))
  } else {
    Ok(bytes)
  }
}
// Try to parse the body of the request as json into object of type T
// T to parse into is set to what you save the return value into
pub async fn parse_json<T: DeserializeOwned>(
  req: &mut Request,
  max_len: usize,
) -> Result<T, Error> {
  // Verify content type
  let content_type = get_header(req, "Content-Type")?.unwrap_or("");
  if "application/json; charset=utf-8" != content_type {
    return Err(Error::invalid_content_type(
      "application/json; charset=utf-8",
      content_type,
    ));
  }
  // Get body
  let bytes = get_body(req, max_len).await?;
  // Try to parse
  let data: T = serde_json::from_slice(&bytes)?;
  Ok(data)
}
// Try to parse the uri query part as urlencoded into object of type T
// T to parse into is set to what you save the return value into
pub fn parse_filter<T: DeserializeOwned>(
  req: &Request,
) -> Result<T, Error> {
  let query_str = req.uri().query().unwrap_or("");
  let filter: T = serde_urlencoded::from_str(query_str)?;
  Ok(filter)
}
