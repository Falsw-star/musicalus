use axum::{Router, routing};
use li_logger::get_logger;
use musicalus::CONFIG;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_handle_def = li_logger::async_init(100, li_logger::default_formatter);
    let (log_handle_mid, middle_logger) = li_logger::middleware::middleware(
        100, li_logger::middleware::default_formatter);
    let mut logger = get_logger();

    let router = Router::new()
        .nest("/netease", musicalus::netease_music::router())
        .route("/", routing::any(|| async { "Hello, World!" }))
        .layer(middle_logger);

    let host = format!("{}:{}", CONFIG.host, CONFIG.port);
    logger.strong().info(format!("Launching server at {host}"));
    let listener = TcpListener::bind(host).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await.ok();
            get_logger().strong().info("Server exiting...");
        }).await?;
    
    logger.strong().success("Done.");
    li_logger::close();
    log_handle_def.await?;
    log_handle_mid.await?;
    Ok(())
}