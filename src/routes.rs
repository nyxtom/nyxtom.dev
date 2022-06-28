use serde_json::json;
use tide::{Request, Response, StatusCode};
use tracing::Instrument;

use crate::{post::Post, registry::REGISTRY};

pub fn configure(app: &mut tide::Server<()>) {
    app.at("/").get(index);
    app.at("/health_check").get(health_check);
    app.at("/about").get(about);
    app.at("/todo").get(todo);
    app.at("/posts/:year/:month/:day/:id").get(get_post);
}

async fn render_markdown(url: &str) -> tide::Result<Response> {
    let post = Post::from_file(url).await?;
    REGISTRY.with(|c| c.render("post.html", &json!(post)))
}

// Returns a simple 200 OK response
async fn health_check(_req: Request<()>) -> tide::Result<Response> {
    Ok(Response::new(StatusCode::Ok))
}

/// Renders the index markdown root file
async fn index(_req: Request<()>) -> tide::Result<Response> {
    render_markdown("posts/index.md").await
}

/// Renders the about markdown root file
async fn about(_req: Request<()>) -> tide::Result<Response> {
    render_markdown("posts/about.md").await
}

/// Renders the todo markdown root file
async fn todo(_req: Request<()>) -> tide::Result<Response> {
    render_markdown("posts/todo.md").await
}

/// Renders a post based on the given path
async fn get_post(req: Request<()>) -> tide::Result<Response> {
    // open up file based on request (fallback to not found)
    let url = format!(
        "posts/{}/{}/{}/{}.md",
        req.param("year")?,
        req.param("month")?,
        req.param("day")?,
        req.param("id")?
    );

    let span = tracing::info_span!("rendering markdown");
    render_markdown(&url).instrument(span).await
}
