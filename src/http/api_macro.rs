pub trait HttpTypeProvider {
    type Adapter;
    type UseCase;
}

#[macro_export]
macro_rules! actix_api {
    ($name:ident) => {
        pub async fn $name(
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
