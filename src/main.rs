use std::sync::Arc;

use crate::app_ctx::AppContext;

mod app_ctx;
mod http_server;

#[tokio::main]
async fn main() {
    let app = AppContext::new();
    let app = Arc::new(app);

    crate::http_server::build_and_start(&app);

    app.states.wait_until_shutdown().await;
}
