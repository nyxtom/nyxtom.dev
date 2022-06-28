use serde_json::json;
use std::io::ErrorKind;
use tide::{Response, StatusCode};

use crate::registry::REGISTRY;

pub async fn error_handler(mut res: Response) -> tide::Result<Response> {
    if let Some(err) = res.downcast_error::<async_std::io::Error>() {
        if let ErrorKind::NotFound = err.kind() {
            res.set_status(StatusCode::NotFound);
        }
    }
    let status = res.status();
    if !status.is_success() {
        REGISTRY.with(|c| {
            c.render_body(
                &mut res,
                "post.html",
                &json!({ "content": format!("{} {}", status as u16, status.canonical_reason()) }),
            );
        });
    }
    Ok(res)
}
