use actix_web::ResponseError;
use std::error::Error;
use std::fmt;

pub struct ActixError<T: Error>(pub T);

impl<T: Error> fmt::Debug for ActixError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl<T: Error> fmt::Display for ActixError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl<T: Error> ResponseError for ActixError<T> {}
