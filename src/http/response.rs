use std::borrow::Cow;

use http::{HeaderMap, HeaderValue, StatusCode, Version};

pub trait ApiResponse {
    type Body;

    fn version(&self) -> Version {
        Version::HTTP_2
    }

    fn status(&self) -> StatusCode {
        StatusCode::OK
    }

    fn headers(&self) -> Option<&HeaderMap<HeaderValue>>;

    fn body(&self) -> &Self::Body;

    fn into_parts(self) -> (Head, Self::Body);
}

pub struct Head {
    pub status: StatusCode,
    pub version: Version,
    pub headers: Option<HeaderMap<HeaderValue>>,
}

#[derive(serde::Serialize)]
pub struct HttpResponseBodyTemplate<T> {
    pub code: u32,
    #[serde(flatten)]
    pub body: DataOrError<T>,
}

#[derive(serde::Serialize)]
pub enum DataOrError<T> {
    #[serde(rename = "data")]
    Data(T),
    #[serde(rename = "error")]
    Error(Cow<'static, str>),
}

#[cfg(feature = "actix-web")]
pub mod actix_impl {
    use std::str::FromStr;

    use crate::http::response::Head;

    use super::ApiResponse;

    /// TODO: Remove this helper trait after actix-web 5.0 is released
    /// See: https://github.com/actix/actix-web/issues/3384
    trait HttpToLegacy {
        type Legacy;

        fn to_legacy(self) -> Self::Legacy;
    }

    impl HttpToLegacy for http::Version {
        type Legacy = actix_web::http::Version;

        fn to_legacy(self) -> Self::Legacy {
            if self == http::Version::HTTP_09 {
                actix_web::http::Version::HTTP_09
            } else if self == http::Version::HTTP_10 {
                actix_web::http::Version::HTTP_10
            } else if self == http::Version::HTTP_11 {
                actix_web::http::Version::HTTP_11
            } else if self == http::Version::HTTP_2 {
                actix_web::http::Version::HTTP_2
            } else if self == http::Version::HTTP_3 {
                actix_web::http::Version::HTTP_3
            } else {
                unreachable!()
            }
        }
    }

    impl HttpToLegacy for http::StatusCode {
        type Legacy = actix_web::http::StatusCode;

        fn to_legacy(self) -> Self::Legacy {
            actix_web::http::StatusCode::from_u16(self.as_u16()).unwrap()
        }
    }

    pub trait ToActixResponse {
        fn to_actix_response(self) -> actix_web::HttpResponse;
    }

    impl<T> ToActixResponse for T
    where
        T: ApiResponse,
        T::Body: serde::Serialize,
    {
        fn to_actix_response(self) -> actix_web::HttpResponse {
            let response = self;
            let (head, body) = response.into_parts();
            let Head {
                status,
                version,
                headers,
            } = head;
            let template = crate::http::response::HttpResponseBodyTemplate {
                code: 0,
                body: crate::http::response::DataOrError::Data(body),
            };
            let mut response = actix_web::HttpResponse::build(status.to_legacy()).json(template);
            if let Some(headers) = headers {
                let resp_headers = response.headers_mut();
                for (k, v) in headers.iter() {
                    let k = actix_web::http::header::HeaderName::from_str(k.as_str()).unwrap();
                    let v = actix_web::http::header::HeaderValue::from_bytes(v.as_bytes()).unwrap();
                    resp_headers.append(k, v);
                }
            }
            response.head_mut().version = version.to_legacy();
            response.head_mut().status = status.to_legacy();

            response
        }
    }
}
