mod errors;
mod post;
mod registry;
mod routes;

use tide::utils::After;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut app = tide::new();
    tide::log::start();

    // serve static files
    app.at("/static").serve_dir("client/dist")?;
    app.at("/assets").serve_dir("posts/assets")?;
    app.at("/favicon.ico").serve_file("favicon.ico")?;

    app.with(tide::log::LogMiddleware::new());
    app.with(After(errors::error_handler));
    routes::configure(&mut app);

    // listen and await
    app.listen("127.0.0.1:1234").await?;
    Ok(())
}
