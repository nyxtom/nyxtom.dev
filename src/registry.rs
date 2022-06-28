use handlebars::Handlebars;
use serde::Serialize;
use tide::Body;
use tide::Response;

#[derive(Clone)]
pub struct State {
    registry: Handlebars<'static>,
}

impl State {
    pub fn default() -> Self {
        let mut state = State {
            registry: Handlebars::new(),
        };
        state.template("post.html", "client/dist/post.html");
        state
    }

    pub fn template(&mut self, name: &str, path: &str) {
        self.registry.register_template_file(name, path).unwrap();
    }

    /// Renders a simple response given serialized data and a template name.
    ///
    /// ## Examples
    ///
    /// ```
    /// use tide::{Response, StatusCode};
    /// use serde_json::json;
    /// use crate::registry::State;
    ///
    /// let state = State::default();
    /// state.render("post.html", &json!({ "content": "hello world" }));
    /// ```
    pub fn render<T: Serialize>(&self, name: &str, data: &T) -> tide::Result<Response> {
        let mut response = Response::new(200);
        self.render_body(&mut response, name, data);
        Ok(response)
    }

    /// Renders a body of content to a response type with the serialized data
    ///
    /// ## Examples
    ///
    /// ```
    /// use tide::{Response, StatusCode};
    /// use serde_json::json;
    /// use crate::registry::State;
    ///
    /// let state = State::default();
    /// let mut response = Response::new(StatusCode::Ok);
    /// state.render_body(response, "post.html", &json!({ "content": "hello world" }));
    /// ```
    pub fn render_body<T: Serialize>(&self, response: &mut Response, name: &str, data: &T) {
        let body = self.registry.render(name, data).unwrap();
        let mut body = Body::from_string(body);
        body.set_mime("text/html");
        response.set_body(body);
    }
}

thread_local! {
    pub static REGISTRY: State = State::default();
}
