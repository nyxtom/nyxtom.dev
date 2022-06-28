use async_std::{fs::File, io::ReadExt};
use serde::Serialize;
use std::{collections::VecDeque, io::Result};

#[derive(Serialize, Default)]
pub struct Post {
    slug: String,
    url: String,
    title: String,
    description: String,
    content: String,
}

impl Post {
    pub fn new() -> Self {
        Post {
            ..Default::default()
        }
    }

    pub async fn from_file(path: &str) -> Result<Self> {
        // open markdown file and read to string
        tracing::info!("reading markdown file path {}", path);
        let url = path.strip_suffix(".md").unwrap();
        let mut md_file = File::open(path).await?;
        let mut buf = String::new();
        md_file.read_to_string(&mut buf).await?;

        let mut post = Post::new();
        post.url = String::from(url);
        post.content = buf;

        if post.content.starts_with("---\n") {
            let mut results: VecDeque<&str> = post.content.splitn(3, "---\n").skip(1).collect();
            let vars = results.pop_front().unwrap();
            let content = results.pop_front().unwrap();

            tracing::info!("variables declared in markdown {}", vars);
            for line in vars.lines() {
                let (k, v) = line.split_once(":").unwrap();
                let v = String::from(v.trim());
                match k {
                    "title" => post.title = v,
                    "description" => post.description = v,
                    "slug" => post.slug = v,
                    _ => {}
                };
            }

            post.content = String::from(content);
        }

        // convert markdown file to html
        tracing::debug!("parsing markdown into html {}", post.content);
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);
        let parser = pulldown_cmark::Parser::new_ext(&post.content, options);
        let mut html_content = String::new();
        pulldown_cmark::html::push_html(&mut html_content, parser);
        post.content = String::from(html_content);

        Ok(post)
    }
}
