#[derive(thiserror::Error, Debug)]
pub enum NinoverseHttpHandlerError {
    #[error("TCP_LISTENER: Error writing in buffer.")]
    BufferError {
        additional_info: String
    },
    #[error("TCP_LISTENER: Error handling the TcpStream.")]
    StreamError {
        additional_info: String
    },
    #[error("TCP_LISTENER: Error parsing a response.")]
    ParsingError {
        additional_info: String
    },
    #[error("TCP_LISTENER: Error building the request struct.")]
    RequestStructError {
        additional_info: String
    },
    #[allow(dead_code)]
    #[error("TCP_LISTENER: Error building the response struct.")]
    ResponseStructError {
        additional_info: String
    }
}