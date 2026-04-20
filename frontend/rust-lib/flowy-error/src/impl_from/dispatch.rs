use bytes::Bytes;

use lib_dispatch::prelude::{AFPluginEventResponse, ResponseBuilder};

use crate::FlowyError;

impl lib_dispatch::Error for FlowyError {
  fn as_response(&self) -> AFPluginEventResponse {
    // Temporarily serialize as JSON since ProtoBuf derive is disabled (Phase 2B)
    let json_str = format!(r#"{{"code":{},"msg":"{}","payload":"{}"}}"#, 
      self.code.clone() as u8,
      self.msg.replace('"', "\\\""),
      String::from_utf8_lossy(&self.payload).replace('"', "\\\"")
    );
    let bytes = Bytes::from(json_str);
    ResponseBuilder::Err().data(bytes).build()
  }
}

// Implement TryFrom for temporary ProtoBuf compatibility
impl std::convert::TryFrom<FlowyError> for Bytes {
  type Error = std::convert::Infallible;
  
  fn try_from(error: FlowyError) -> Result<Self, Self::Error> {
    let json_str = format!(r#"{{"code":{},"msg":"{}","payload":"{}"}}"#, 
      error.code as u8,
      error.msg.replace('"', "\\\""),
      String::from_utf8_lossy(&error.payload).replace('"', "\\\"")
    );
    Ok(Bytes::from(json_str))
  }
}
