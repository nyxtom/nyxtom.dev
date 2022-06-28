# nyxtom.dev

This is my personal blog implemented in #rust using the tide web framework, async-std, http-types, async-h1 and pulldown-cmark for markdown commonmark spec support. I've used serde to serialize simple data and I have a few scripts to re-run the indexing for the blog. It's meant to be a straightforward implementation of a markdown based blog while using Rust. I've written up a post on how to implement this on the blog here as well:

- [2022-06-25 Markdown Blog in Rust](https://nyxtom.dev/posts/2022/06/25/tide)

Thanks for checking it out!

## Implementing a Blog

Now that we've got the fundamental dependencies covered, let's go ahead and actually make use of them with [tide](https://github.com/http-rs/tide). I highly recommend browsing the above dependencies in detail as it gives a look at how to implement your own runtime, separate concerns, building a protocol layer, and making use of async I/O with single-purpose packages. These are great codebases to learn from and they do well to at aiming towards good ergonomics and idiomatics. I imagine more libraries will get similarly clean as new features are introduced in the rust language with support for things like [async traits](https://rust-lang.github.io/wg-async/vision/roadmap/async_iter/traits.html) and even [portable/interopable runtimes](https://rust-lang.github.io/wg-async/vision/roadmap/portable/runtime.html).

In any case, let's move along and build that blog! While the rest of the dependencies certainly aren't under 100 LOC, they are great learning resources. Our blog implementation will make use of these dependencies to deliver a minimal codebase.

### Setup

As before, we will use the dependencies listed at the beginning of this post. Here it is once more:

```toml
[package]
name = "notes"
version = "0.1.0"
edition = "2021"

[dependencies]
async-std = { version = "1.12.0", features = ["attributes"] }
handlebars = "4.3.1"
pulldown-cmark = "0.9.1"
serde = { version = "1.0.137", features = ["derive"] }
serde_json = "1.0.81"
tide = "0.16.0"
```

Our main function will setup a few modules we will expand on and import to run the **tide** app.

```rust
mod registry;
mod routes;
mod errors;

use tide::utils::After;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut app = tide::new();
    // serve static files
    app.at("/static").serve_dir("client/dist")?;

    app.with(After(errors::error_handler));
    routes::configure(&mut app);

    // listen and await
    app.listen("127.0.0.1:1234").await?;
    Ok(())
}
```

The use of [tide::utils::After](https://docs.rs/tide/latest/tide/utils/struct.After.html) is a middleware that operates on outgoing responses. This is useful for us as we want to be able to have a catch-all for errors and write the appropriate response for them.

### Template Registry

In order to support simple templates that output markdown html we will use the [handlebars](https://docs.rs/handlebars) crate to get the job done. Within the `registry.rs` file as specified in the `mod registry` declaration add the following:

```rust
use handlebars::Handlebars;

#[derive(Clone)]
pub struct State {
    registry: Handlebars<'static>,
}

impl State {
    pub fn default() -> Self {
        let mut state = State {
            registry: Handlebars::new(),
        };
        state.registry.set_dev_mode(true);
        state.template("index.html", "client/dist/index.html");
        state.template("posts.html", "client/dist/posts.html");
        state
    }

    pub fn template(&mut self, name: &str, path: &str) {
        self.registry.register_template_file(name, path).unwrap();
    }
}
```

> [handlebars::Handlebars::set_dev_mode](https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.set_dev_mode) is a useful mode to allow us to ensure that changes made to the template will be loaded on every request rather than cached.

The [handlebars::Handlebars::register_template_file](https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.register_template_file) here is simply registering a key to a file source based on the passed in path and name. This is later used below with a call to [render](https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.render) where the compiled [handlebars::template::Template](https://docs.rs/handlebars/latest/handlebars/template/struct.Template.html) is evaluated with the provided serializable context.

In order to render responses to [http-types::Response](https://docs.rs/http-types/latest/http_types/struct.Response.html) that can then be encoded by [async-h1](https://docs.rs/async-h1) let's create a few utility methods.

```rust
use serde::Serialize;
use tide::{Body, Response};

impl State {
    pub fn default() -> Self { ... }
    pub fn template(&mut self, name: &str, path: &str) { ... }

    pub fn render<T: Serialize>(&self, name: &str, data: &T) -> tide::Result<Response> {
        let mut response = Response::new(200);
        self.render_body(&mut response, name, data);
        Ok(response)
    }

    pub fn render_body<T: Serialize>(&self, response: &mut Response, name: &str, data: &T) {
        let body = self.registry.render(name, data).unwrap();
        let mut body = Body::from_string(body);
        body.set_mime("text/html");
        response.set_body(body);
    }
}
```

Following the use of the [handlebars::Handlebars::render](https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.render), we also construct an [http-types::Body](https://docs.rs/http-types/latest/http_types/struct.Body.html) from the rendered content and set the mime type header before returning the [http-types:Response](https://docs.rs/http-types/latest/http_types/struct.Response.html). Finally, we are going to make use of a [thread static local](https://doc.rust-lang.org/stable/std/macro.thread_local.html) state to encapsulate the template registry (rather than use the **tide::State** as part of the [Request](https://docs.rs/tide/latest/tide/struct.Request.html#method.state). The main reason for this is that I wanted to be able to use this static template registry in the error handling middleware as well without having any effect on the request pipeline. There are other ways we could do this but for now I wanted to keep template registry as a pre-built static registry. Simply add the following to the `registry.rs` file below:

```rust
thread_local! {
    pub static REGISTRY: State = State::default();
}
```

### Routes

In order to handle a few handlers for the blog we simply need to register the route we are interested in such as `posts/:year/:month/:day/:title` to capture each of the variables in the request parameters. We will make quick use of [pulldown-cmark](https://docs.rs/pulldown_cmark) to transform a loaded [async_std::fs::File](https://docs.rs/async-std/latest/async_std/fs/struct.File.html) into HTML output through the [pulldown_cmark::Parser](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Parser.html)

```rust
use async_std::{fs::File, io::ReadExt};
use serde_json::json;

use crate::registry::REGISTRY;

pub fn configure(app: &mut tide::Server<()>) {
    app.at("/posts/:year/:month/:day/:id").get(get_post);
}

async fn get_post(req: tide::Request<()>) -> tide::Result<tide::Response> {
    // open up file based on request (fallback to not found)
    let url = format!(
        "posts/{}/{}/{}/{}.md",
        req.param("year")?,
        req.param("month")?,
        req.param("day")?,
        req.param("id")?
    );
    // open markdown file and read to string
    let mut md_file = File::open(url).await?;
    let mut buf = String::new();
    md_file.read_to_string(&mut buf).await?;

    // convert markdown file to html
    let parser = pulldown_cmark::Parser::new(&buf);
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    // render template with content
    REGISTRY.with(|c| c.render("index.html", &json!({ "content": html_content })))
}
```

Finally, a call to the *REGISTRY* to **render** the provided template and pass along some [json!](https://docs.rs/serde_json/latest/serde_json/macro.json.html) data thanks to [serde-json](https://docs.rs/serde_json).

### Error Handling

As a fallback for when we encounter errors (such as when a file is not found or we receive an invalid request) we declare the use of `After(errors::error_handler)`. Go ahead and add the following to `errors.rs`.

```rust
use std::io::ErrorKind;
use serde_json::json;

use crate::registry::REGISTRY;

async fn error_handler(mut res: tide::Response) -> tide::Result<Response> {
    if let Some(err) = res.downcast_error::<async_std::io::Error>() {
        if let ErrorKind::NotFound = err.kind() {
            res.set_status(StatusCode::NotFound);
        }
    }
    if res.status() == StatusCode::NotFound {
        REGISTRY.with(|c| {
            c.render_body(&mut res, "index.html", &json!({ "content": "Not found" }));
        });
    }
    Ok(res)
}
```

Here we are making use of [std::io::ErrorKind](https://doc.rust-lang.org/stable/std/io/enum.ErrorKind.html) to determine if the error happens to be a downcast from an *async-std::io::Error* and happens to be a **NotFound** error kind. We use this to set the appropriate [set_status](https://docs.rs/tide/latest/tide/struct.Response.html#method.set_status) on the response. Finally, assuming the status code is **NotFound** we will simply render out the standard **index.html** template with some *Not Found* content.


### Tailwind + Parcel + Highlight.js

Now that the backend is completed in Rust, we can move along to the HTML templates. I've chosen to use [parcel](https://parceljs.org/) as the build tool, [tailwindcss](https://tailwindcss.com/) as the CSS framework, and [highlight.js](https://highlightjs.org/) for code syntax highlighting. We've already implemented the markdown to HTML part of the application, we simply need a way to render it in HTML/CSS. In a new directory `client` run the following.

```bash
mkdir client
touch client/package.json
```

#### Dependencies

Edit `package.json` and add the following, followed by running `npm install` to ensure the dependencies are properly setup.

```json
{
  "name": "notes",
  "source": "src/index.html",
  "scripts": {
    "start": "parcel",
    "build": "parcel build"
  },
  "targets": {
      "default": {
          "publicUrl": "/static"
      }
  },
  "devDependencies": {
    "@tailwindcss/typography": "^0.5.2",
    "parcel": "latest",
    "postcss": "^8.4.14",
    "tailwindcss": "^3.1.4"
  }
}
```

#### Index Template / Tailwind CSS

Now that we have the dependencies in order and a few scripts we can setup the `client/src/index.html` and `client/src/index.css` appropriately.

```html
<!DOCTYPE HTML>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Hello World</title>
    <link href="./index.css" rel="stylesheet">
    <link rel="stylesheet" href="https://unpkg.com/@highlightjs/cdn-assets@11.5.1/styles/default.min.css">
</head>
<body class="container mx-auto bg-slate-50 py-4 antialiased">
    <div class="shadow-md border-t-4 bg-white rounded-md">
        <nav class="p-8 flex text-sm text-sky-900 lowercase tracking-wide">
            <h1 class="flex-initial font-medium uppercase"><a href="/">✎ Notes</a></h1>
            <div class="flex-1"></div>
            <div class="text-xs font-semibold">
                <a href="/todo">todo!</a>
            </div>
        </nav>
        <article class="p-8 prose lg:prose-l max-w-full">
            {{{content}}}
        </article>
    </div>
    <footer class="flex py-5 px-3 text-xs font-bold text-gray-300 lowercase tracking-wide">
        <span>@nyxtom | <span class="italic">#tailwind #rustlang</span></span>
        <div class="flex-1"></div>
        <a href="https://twitter.com/nyxtom" class="text-gray-400 hover:text-gray-800 dark:hover:text-white">
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path d="M8.29 20.251c7.547 0 11.675-6.253 11.675-11.675 0-.178 0-.355-.012-.53A8.348 8.348 0 0022 5.92a8.19 8.19 0 01-2.357.646 4.118 4.118 0 001.804-2.27 8.224 8.224 0 01-2.605.996 4.107 4.107 0 00-6.993 3.743 11.65 11.65 0 01-8.457-4.287 4.106 4.106 0 001.27 5.477A4.072 4.072 0 012.8 9.713v.052a4.105 4.105 0 003.292 4.022 4.095 4.095 0 01-1.853.07 4.108 4.108 0 003.834 2.85A8.233 8.233 0 012 18.407a11.616 11.616 0 006.29 1.84" /></svg>
        </a>
        <a href="https://youtube.com/c/nyxtom" class="pl-2 text-gray-400 hover:text-gray-800 dark:hover:text-white">
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
        </a>
    </footer>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
    <script type="module">
        hljs.highlightAll();
    </script>
</body>
</html>
```

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
    .prose pre {
        padding: 0px;
        background: none;
        @apply border-t-2 border-t-gray-50 rounded-md block shadow-md;
    }
}
```

These files make use of a number of tailwind utilities but don't be particularly intimidated by them. A lot of that is just padding, the use of [tailwind/typography](https://tailwindcss.com/docs/typography-plugin) and various flexbox and color values. I've also added a few social svg icons in the footer and styles for the font to make it a bit easier on the eye. The bulk of the work is being done by [highlight.js](https://highlightjs.org) to perform the actual syntax highlighting by the code. 

Also make note of the `{{{content}}}` in the template. This is a **raw html** declaration that simply passes along the variable *content* as we declared in the rust backend code. Handlebars will make sure to parse this and render that content appropriately.

#### Tailwind Configuration

In order for tailwind to take effect, you'll also need a `tailwind.config.js` configuration file so that **parcel** can actually build the content. You'll also need a simple `.postcssrc` for postcss to work in combination with tailwind.

```js
module.exports = {
    content: [
        "./src/**/*.{html,css,js,jsx,ts,tsx}"
    ],
    theme: {
        container: {
            center: true
        },
        extend: {}
    },
    plugins: [
        require("@tailwindcss/typography")
    ],
}
```

```
{
    "plugins": {
        "tailwindcss": {}
    }
}
```
