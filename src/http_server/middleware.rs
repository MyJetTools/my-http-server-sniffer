use std::sync::Arc;

use flurl::{
    FlUrl,
    hyper::{self, Method},
};
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpRequestHeaders,
    HttpServerMiddleware, my_hyper_utils::ToMyHttpResponse,
};

pub struct MyMiddleware {
    remote_url: String,
}

impl MyMiddleware {
    pub fn new() -> Self {
        let mut remote_url = match std::env::var("REMOTE_URL") {
            Ok(value) => value,
            Err(_) => "https://jetdev.eu".to_string(),
        };

        if remote_url.chars().last().unwrap() == '/' {
            remote_url.pop();
        }

        Self { remote_url }
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for MyMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let response = match ctx.request.method {
            Method::GET => {
                let url = format!("{}{}", self.remote_url, ctx.request.get_path_and_query());

                println!("Url: {}", url);

                let mut fl_url = FlUrl::new(url);

                for (key, value) in ctx.request.get_headers().to_hash_map() {
                    if !key.eq_ignore_ascii_case("host") {
                        fl_url = fl_url.with_header(key, value);
                    }
                }

                fl_url.get().await
            }
            Method::POST => {
                let url = format!("{}{}", self.remote_url, ctx.request.get_path_and_query());
                let mut fl_url = FlUrl::new(url);

                for (key, value) in ctx.request.get_headers().to_hash_map() {
                    fl_url = fl_url.with_header(key, value);
                }

                let body = ctx.request.get_body().await.unwrap().as_slice();

                println!("{:?}", std::str::from_utf8(body));
                fl_url.post(Some(body.to_vec())).await
            }

            _ => {
                return None;
            }
        };

        if let Err(err) = &response {
            println!("{:?}", err);
            return Some(Err(HttpFailResult::as_fatal_error(format!("{:?}", err))));
        }

        let response = response.unwrap();

        let mut builder = hyper::Response::builder().status(response.get_status_code());

        for (key, value) in response.get_headers() {
            if let Some(v) = value {
                builder = builder.header(key.to_string(), v.to_string());
            }
        }

        let body = response.receive_body().await.unwrap();

        let response = (builder, body).to_my_http_response();

        Some(HttpOutput::Raw(response).into_ok_result(false))
    }
}
