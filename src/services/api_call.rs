use crate::services::api::ApiError;

pub trait ApiCall {
    fn new() -> Self where Self: Sized {
        unimplemented!()
    }
    fn set_url(&mut self, _url: &str) -> &mut Self {
        self
    }
    fn send(&mut self) -> Result<Option<String>, ApiError> {
        unimplemented!()
    }
} 
