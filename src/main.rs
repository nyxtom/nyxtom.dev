mod errors;
mod post;
mod registry;
mod routes;

use tide::utils::After;
use tide_tracing::TraceMiddleware;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut app = tide::new();
    // env_logger::init();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // serve static files
    app.at("/static").serve_dir("client/dist")?;
    app.at("/assets").serve_dir("content/assets")?;
    app.at("/favicon.ico").serve_file("favicon.ico")?;

    // app.with(tide::log::LogMiddleware::new());
    app.with(TraceMiddleware::new());
    app.with(After(errors::error_handler));
    routes::configure(&mut app);

    // listen and await
    let host = option_env!("HOST").unwrap_or("0.0.0.0");
    let port = option_env!("PORT").unwrap_or("7000");
    app.listen(format!("{}:{}", host, port)).await?;
    Ok(())
}
