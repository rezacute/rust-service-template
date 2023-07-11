
use actix_web::{
    dev::Service,
    http::header::{HeaderName, HeaderValue},
    web, App, HttpMessage, HttpServer,HttpResponse
};
use tonic::codegen::http::response;
use tracing::info;
use tracing_actix_web::{RequestId, TracingLogger};
use tracing_subscriber;

use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};
use serde::{Serialize, Deserialize};
#[derive(OpenApi)]
#[openapi(
    info(description = "My Api description"),
)]
pub struct ApiDoc;

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

async fn hello() -> HttpResponse {
    let response = HelloResponse {
        message: "Hello world!".to_string(),
    };

    HttpResponse::Ok().json(response)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the tracing subscriber with the desired filter level
    use std::env;


    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
    let lvl = match log_level.as_str() {
        "TRACE" => tracing::Level::TRACE,
        "DEBUG" => tracing::Level::DEBUG,
        "INFO" => tracing::Level::INFO,
        "WARN" => tracing::Level::WARN,
        "ERROR" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    tracing_subscriber::fmt()
        .with_max_level(lvl)
        .init();
    let address = std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    info!("Starting server at {}", address);
    HttpServer::new(move || {
        App::new()
            // set the request id in the `x-request-id` response header
            .wrap_fn(|req, srv| {
                let request_id = req.extensions().get::<RequestId>().copied();
                let res = srv.call(req);
                async move {
                    let mut res = res.await?;
                    if let Some(request_id) = request_id {
                        res.headers_mut().insert(
                            HeaderName::from_static("x-request-id"),
                            // this unwrap never fails, since UUIDs are valid ASCII strings
                            HeaderValue::from_str(&request_id.to_string()).unwrap(),
                        );
                    }
                    Ok(res)
                }
            })
            .wrap(TracingLogger::default())
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                (
                    Url::new("hello", "/api-doc/openapi_auth.json"),
                    ApiDoc::openapi(),
                ),
            ]))
            .service(web::resource("/hello").to(hello))
    })
    .bind(address)?
    .run()
    .await?;
    Ok(())
}
