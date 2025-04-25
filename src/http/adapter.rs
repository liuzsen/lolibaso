use crate::use_case::UseCase;

use super::{error::BizError, request::HttpRequest};

pub trait Adapter<U: UseCase, P> {
    type Response;

    fn convert_input<R>(&self, request: R, parser: P) -> Result<U::Input, BizError>
    where
        R: HttpRequest;

    fn convert_output(&self, output: U::Output) -> Self::Response;

    fn convert_err(&self, err: U::Error) -> BizError;
}
