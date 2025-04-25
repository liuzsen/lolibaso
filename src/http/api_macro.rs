#[macro_export]
macro_rules! actix_api {
    ($name:ident, $path0:ident $(::$path_seg:ident)*) => {
        pub async fn $name(
            req: actix_web::HttpRequest,
            payload: actix_web::web::Bytes,
        ) -> Result<actix_web::HttpResponse, lolibaso::http::error::HttpError> {
            use lolibaso::http::Adapter;
            use lolibaso::http::request::actix_impl::ActixHttpRequest;
            use lolibaso::provider::Provider;
            use lolibaso::use_case::UseCase;

            type __UseCase = crate::infratructure::types:: $path0 $(::$path_seg)* ::UseCase;
            type __JsonParser = crate::infratructure::types::JsonParser;

            let req = ActixHttpRequest {
                request: req,
                payload,
            };
            // get adapter
            let adapter = crate::adapters::api_http:: $path0 $(::$path_seg)* ::Adapter::new();

            // convert input
            let input = Adapter::<__UseCase, _>::convert_input(&adapter, req, __JsonParser::new())?;

            // do use case
            let use_case = __UseCase::provide()?;
            let output = use_case.execute(input).await?;

            match output {
                Ok(output) => {
                    // convert output
                    let response = Adapter::<__UseCase, __JsonParser>::convert_output(&adapter, output);
                    Ok(lolibaso::http::response::actix_impl::ToActixResponse::to_actix_response(response))
                },
                Err(err) => {
                    // convert error
                    let err = Adapter::<__UseCase, __JsonParser>::convert_err(&adapter, err);
                    return Err(From::from(err));
                }
            }
        }
    };
}
