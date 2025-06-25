use std::ops::Deref;

use super::error::BizError;

pub struct Request<T>(http::Request<T>);

impl<T> Deref for Request<T> {
    type Target = http::Request<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait HttpRequest {
    fn method(&self) -> http::Method;

    fn uri(&self) -> &http::Uri;

    fn version(&self) -> http::Version;

    fn headers(&self) -> http::HeaderMap<http::HeaderValue>;

    fn body(&self) -> Option<&[u8]>;
}

pub trait FromRequest<R, P>: Sized {
    fn from_request(request: R, parser: P) -> Result<Self, BizError>;
}

#[cfg(feature = "actix-web")]
pub mod actix_impl {
    use std::str::FromStr;

    use actix_web::web::Bytes;

    use super::*;

    pub struct ActixHttpRequest {
        pub request: actix_web::HttpRequest,
        pub payload: Bytes,
        // 缓存转换后的 URI，避免重复转换，主要为了解决 http crate 版本的问题
        // 当 actix-web 发布 5.0 后应该删除这个字段
        cached_uri: std::sync::OnceLock<http::Uri>,
    }

    impl ActixHttpRequest {
        pub fn new(request: actix_web::HttpRequest, payload: Bytes) -> Self {
            Self {
                request,
                payload,
                cached_uri: std::sync::OnceLock::new(),
            }
        }
    }

    impl HttpRequest for ActixHttpRequest {
        fn method(&self) -> http::Method {
            let s = self.request.method().as_str();
            http::Method::from_str(s).unwrap()
        }

        fn uri(&self) -> &http::Uri {
            // 获取 URI 的引用，安全地将 actix-web 的 URI 转换为 http v1.x 的 URI
            self.cached_uri.get_or_init(|| {
                let uri_str = self.request.uri().to_string();
                uri_str.parse().expect("Invalid URI")
            })
        }

        fn version(&self) -> http::Version {
            let version = self.request.version();
            if version == actix_web::http::Version::HTTP_10 {
                http::Version::HTTP_10
            } else if version == actix_web::http::Version::HTTP_11 {
                http::Version::HTTP_11
            } else if version == actix_web::http::Version::HTTP_2 {
                http::Version::HTTP_2
            } else if version == actix_web::http::Version::HTTP_3 {
                http::Version::HTTP_3
            } else {
                panic!("Unsupported HTTP version: {:?}", version)
            }
        }

        fn headers(&self) -> http::HeaderMap<http::HeaderValue> {
            let headers = self.request.headers();
            let mut res = http::HeaderMap::new();
            for (key, value) in headers.into_iter() {
                let key = http::header::HeaderName::from_bytes(key.as_str().as_bytes()).unwrap();
                let value = http::HeaderValue::from_bytes(value.as_bytes()).unwrap();
                res.insert(key, value);
            }
            res
        }

        fn body(&self) -> Option<&[u8]> {
            Some(self.payload.as_ref())
        }
    }
}
