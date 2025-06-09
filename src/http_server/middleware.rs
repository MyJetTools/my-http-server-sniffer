use std::io::Read;

use flate2::bufread::GzDecoder;
use flurl::{
    FlUrl,
    hyper::{self, Method},
};
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpRequestHeaders,
    HttpServerMiddleware, my_hyper_utils::ToMyHttpResponse,
};
use rust_extensions::base64::IntoBase64;

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
        println!("----- New Request");
        println!(
            "[{}] {}",
            ctx.request.method,
            ctx.request.get_path_and_query()
        );
        println!("Headers:");

        let headers = ctx.request.get_headers().to_hash_map();

        for (key, value) in headers.iter() {
            println!("{}:{}", key, value);
        }

        let response = match ctx.request.method {
            Method::GET => {
                let url = format!("{}{}", self.remote_url, ctx.request.get_path_and_query());

                let mut fl_url = FlUrl::new(url);

                for (key, value) in headers {
                    if !key.eq_ignore_ascii_case("host") {
                        fl_url = fl_url.with_header(key, value);
                    }
                }

                fl_url.get().await
            }
            Method::POST => {
                let url = format!("{}{}", self.remote_url, ctx.request.get_path_and_query());
                let mut fl_url = FlUrl::new(url);

                for (key, value) in headers {
                    if !key.eq_ignore_ascii_case("host") {
                        fl_url = fl_url.with_header(key, value);
                    }
                }

                let body = ctx.request.get_body().await.unwrap().as_slice();

                println!("Request Body Start:");
                match std::str::from_utf8(body) {
                    Ok(body_as_str) => {
                        println!("{:?}", body_as_str);
                    }
                    Err(_) => {
                        println!("{}", body.into_base64());
                    }
                }

                println!("Request Body End:");
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

        println!("Response Status: {}", response.get_status_code());

        let mut builder = hyper::Response::builder().status(response.get_status_code());

        println!("Response Headers:");

        let mut gzip = false;
        for (key, value) in response.get_headers() {
            if let Some(v) = value {
                println!("{}:{}", key, v);

                if key.eq_ignore_ascii_case("content-encoding") {
                    if v.contains("gzip") {
                        gzip = true;
                    }
                }
                builder = builder.header(key.to_string(), v.to_string());
            }
        }

        let mut body_as_stream = response.get_body_as_stream();

        let mut body = Vec::new();

        while let Some(chunk) = body_as_stream.get_next_chunk().await.unwrap() {
            if gzip {
                let mut decoder = GzDecoder::new(chunk.as_slice());

                let mut decompressed = Vec::new();
                decoder.read(&mut decompressed).unwrap();
                body.extend_from_slice(&decompressed);
            } else {
                body.extend_from_slice(&chunk);
            }
        }

        println!("Response Body Start:");
        match std::str::from_utf8(body.as_slice()) {
            Ok(body_as_str) => {
                println!("{:?}", body_as_str);
            }
            Err(_) => {
                println!("{}", body.into_base64());
            }
        }

        println!("Response Body End:");

        println!("----------");

        let response = (builder, body).to_my_http_response();

        Some(HttpOutput::Raw(response).into_ok_result(false))
    }
}
