use std::{net::SocketAddr, sync::Arc};

use my_http_server::MyHttpServer;

use crate::{app_ctx::AppContext, http_server::MyMiddleware};

pub fn build_and_start(app: &Arc<AppContext>) {
    let port = match std::env::var("PORT") {
        Ok(port) => port.parse().unwrap(),
        Err(_) => 8000,
    };
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], port)));

    http_server.add_middleware(Arc::new(MyMiddleware::new()));

    http_server.start(app.states.clone(), my_logger::LOGGER.clone());
}
