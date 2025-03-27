#[derive(thiserror::Error, Debug)]
pub enum NinoverseApiError {
    #[error("Some error occured while handling the api call.")]
    GenericError {
        original_error: String
    },
}