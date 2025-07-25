pub trait HttpTypeProvider {
    type Adapter;
    type UseCase;
}

#[macro_export]
macro_rules! actix_api {
    ($name:ident) => {
        async fn $name(
            req: actix_web::HttpRequest,
            payload: actix_web::web::Bytes,
        ) -> Result<actix_web::HttpResponse, lolibaso::http::error::HttpError> {
            use lolibaso::http::HttpAdapter;
            use lolibaso::http::api_macro::HttpTypeProvider;
            use lolibaso::http::request::actix_impl::ActixHttpRequest;
            use lolibaso::provider::Provider;
            use lolibaso::use_case::UseCase;

            use self::inject::$name;

            type __Parser = crate::infratructure::types::Parser;
            type __UseCase = <$name as HttpTypeProvider>::UseCase;
            type __Adapter = <$name as HttpTypeProvider>::Adapter;

            let req = ActixHttpRequest::new(req, payload);

            // get adapter
            let adapter = __Adapter::new();

            // convert input
            let input =
                HttpAdapter::<__UseCase, _>::convert_input(&adapter, &req, __Parser::provide()?)?;

            // do use case
            let use_case = __UseCase::provide()?;

            // execute use case
            let output = use_case.execute(input).await?;
            match output {
                Ok(output) => {
                    // convert output
                    let response =
                        HttpAdapter::<__UseCase, __Parser>::convert_output(&adapter, output);
                    Ok(
                        lolibaso::http::response::actix_impl::ToActixResponse::to_actix_response(
                            response,
                        ),
                    )
                }
                Err(err) => {
                    // convert error
                    let err = HttpAdapter::<__UseCase, __Parser>::convert_err(&adapter, err);
                    return Err(From::from(err));
                }
            }
        }
    };
}

pub trait WsTypeProvider {
    type Adapter;
    type Command;
    type Event;
}

#[macro_export]
macro_rules! actix_ws_api {
    ($name:ident) => {
        async fn $name(
            req: actix_web::HttpRequest,
            payload: actix_web::web::Payload,
        ) -> Result<actix_web::HttpResponse, lolibaso::http::error::HttpError> {
            use lolibaso::http::request::actix_impl::ActixHttpRequest;
            use lolibaso::http::web_socket::WSAdapter;
            use lolibaso::provider::Provider;
            use lolibaso::http::api_macro::WsTypeProvider;

            use self::inject::$name;

            type __Parser = crate::infratructure::types::Parser;
            type __Chan = crate::infratructure::types::DuplexChanClient<__Command, __Event>;

            type __Adapter = <$name as WsTypeProvider>::Adapter;
            type __Command = <$name as WsTypeProvider>::Command;
            type __Event = <$name as WsTypeProvider>::Event;

            let req_clone = req.clone();
            let mut response = None;
            let get_ws = || {
                let (res, session, stream) =
                    actix_ws::handle(&req_clone, payload).map_err(|err| anyhow::anyhow!("{err}"))?;
                response = Some(res);

                let stream = stream
                    .aggregate_continuations()
                    .max_continuation_size(2_usize.pow(20));
                anyhow::Ok(WebSocketActix::new(session, stream))
            };

            let req = ActixHttpRequest::new(req, Bytes::new());

            let adapter = __Adapter::provide()?;
            let parser = __Parser::provide()?;
            WSAdapter::<__Parser, __Chan>::accept(adapter, &req, parser, get_ws)??;

            match response {
                Some(resp) => Ok(resp),
                None => {
                    panic!(
                        "The implementor of the WSAdapter::accept must either return an error or call `get_ws` to obtain a WebSocketChan"
                    )
                }
            }
        }

    };
}

pub trait QueryRouter {
    type Adapter;
}

pub struct SimpleQueryResponse<T> {
    pub body: T,
}

impl<T> crate::http::ApiResponse for SimpleQueryResponse<T> {
    type Body = T;

    fn headers(&self) -> Option<&http::HeaderMap<http::HeaderValue>> {
        None
    }

    fn body(&self) -> &Self::Body {
        &self.body
    }

    fn into_parts(self) -> (super::response::Head, Self::Body) {
        (
            super::response::Head {
                status: self.status(),
                version: self.version(),
                headers: None,
            },
            self.body,
        )
    }
}

pub trait QueryProvider {
    type Query;
    type Response;

    async fn query(
        &self,
        q: Self::Query,
    ) -> crate::result::BizResult<Self::Response, super::error::BizError>;
}

#[macro_export]
macro_rules! actix_query_api {
    ($name:ident) => {
        const _: () = {
            use lolibaso::http::api_macro::QueryProvider;
            use lolibaso::http::api_macro::QueryRouter;
            use lolibaso::http::api_macro::SimpleQueryResponse;
            use lolibaso::http::response::actix_impl::ToActixResponse;

            impl $name {
                pub async fn query(
                    q: actix_web::web::Query<
                        <<$name as QueryRouter>::Adapter as QueryProvider>::Query,
                    >,
                ) -> Result<actix_web::HttpResponse, lolibaso::http::error::HttpError> {
                    let adapter = <$name as QueryRouter>::Adapter::provide()?;
                    let resp = adapter.query(q.into_inner()).await??;
                    let response = SimpleQueryResponse { body: resp };
                    let response = ToActixResponse::to_actix_response(response);
                    Ok(response)
                }
            }
        };
    };
}
