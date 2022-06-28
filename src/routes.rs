use serde_json::json;
use tide::{Request, Response, StatusCode};

use crate::{post::Post, registry::REGISTRY};

pub fn configure(app: &mut tide::Server<()>) {
    app.at("/").get(index);
    app.at("/health_check").get(health_check);
    app.at("/about").get(about);
    app.at("/todo").get(todo);
    app.at("/posts/:year/:month/:day/:id").get(get_post);
}

async fn render_markdown(url: &str, template: &str) -> tide::Result<Response> {
    let post = Post::from_file(url).await?;
    // render template with content
    REGISTRY.with(|c| c.render(template, &json!(post)))
}

async fn index(_req: Request<()>) -> tide::Result<Response> {
    render_markdown("posts/index.md", "post.html").await
}

async fn health_check(_req: Request<()>) -> tide::Result<Response> {
    Ok(Response::new(StatusCode::Ok))
}

async fn about(_req: Request<()>) -> tide::Result<Response> {
    render_markdown("posts/about.md", "post.html").await
}

async fn todo(_req: Request<()>) -> tide::Result<Response> {
    render_markdown("posts/todo.md", "post.html").await
}

async fn get_post(req: Request<()>) -> tide::Result<Response> {
    // open up file based on request (fallback to not found)
    let url = format!(
        "posts/{}/{}/{}/{}.md",
        req.param("year")?,
        req.param("month")?,
        req.param("day")?,
        req.param("id")?
    );

    render_markdown(&url, "post.html").await
}
