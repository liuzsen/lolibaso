use crate::{
    http::parser::{Format, Parser, UrlEncodedQuery},
    use_case::UseCase,
};

use super::{error::BizError, request::HttpRequest};

pub trait HttpAdapter<'a, U, P>
where
    U: UseCase,
{
    type ReqBodyFormat: Format;
    type Request: HttpRequestModel;
    type Response;

    fn convert_input<R>(&self, request: &'a R, parser: P) -> Result<U::Input, BizError>
    where
        R: HttpRequest,
        P: Parser<'a, <Self::Request as HttpRequestModel>::Query, UrlEncodedQuery>,
        P: Parser<'a, <Self::Request as HttpRequestModel>::Body, Self::ReqBodyFormat>;

    fn convert_output(&self, output: U::Output) -> Self::Response;

    fn convert_err(&self, err: U::Error) -> BizError;
}

pub trait HttpRequestModel {
    type Query;
    type Body;
}

pub trait FromHttpRequest<'a>: HttpRequestModel + Sized {
    fn from_request_<R, P, F>(req: &'a R, parser: P) -> Result<Self, BizError>
    where
        R: HttpRequest,
        F: Format,
        P: Parser<'a, <Self as HttpRequestModel>::Query, UrlEncodedQuery>,
        P: Parser<'a, <Self as HttpRequestModel>::Body, F>;
}
