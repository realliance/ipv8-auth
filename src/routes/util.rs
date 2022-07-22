#[macro_export]
macro_rules! respond {
  ($response_code:expr, $message:expr) => {
    Ok(
      Response::builder()
        .status($response_code)
        .body(Body::from($message))
        .unwrap(),
    )
  };
}
