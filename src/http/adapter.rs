use crate::{
    http::codec::{Format, UrlEncodedQuery, decoder::Decoder},
    use_case::UseCase,
};

use super::{error::BizError, request::HttpRequest};

pub trait HttpAdapter<'a, U, D>
where
    U: UseCase,
{
    type ReqBodyFormat: Format;
    type Request: HttpRequestModel;
    type Response;

    fn convert_input<R>(&self, request: &'a R, decoder: D) -> Result<U::Input, BizError>
    where
        R: HttpRequest,
        D: Decoder<'a, <Self::Request as HttpRequestModel>::Query, UrlEncodedQuery>,
        D: Decoder<'a, <Self::Request as HttpRequestModel>::Body, Self::ReqBodyFormat>;

    fn convert_output(&self, output: U::Output) -> Self::Response;

    fn convert_err(&self, err: U::Error) -> BizError;
}

pub trait HttpRequestModel {
    type Query;
    type Body;
}

impl HttpRequestModel for () {
    type Body = ();
    type Query = ();
}

pub trait FromHttpRequest<'a>: HttpRequestModel + Sized {
    fn from_http_req<R, D, F>(req: &'a R, decoder: &D) -> Result<Self, BizError>
    where
        R: HttpRequest,
        F: Format,
        D: Decoder<'a, <Self as HttpRequestModel>::Query, UrlEncodedQuery>,
        D: Decoder<'a, <Self as HttpRequestModel>::Body, F>;
}
