use suprnova::{InertiaProps, Request, Response, handler, inertia_response};

use crate::markdown;

#[derive(InertiaProps)]
pub struct HomeProps {
    pub title: String,
    pub message: String,
}

/// What the Home page shows. The page and its Markdown twin both read it, so
/// they cannot drift apart.
pub fn content() -> HomeProps {
    HomeProps {
        title: "Welcome to Suprnova!".to_string(),
        message: "Your Inertia app is ready.".to_string(),
    }
}

/// The Home page as Markdown, served at `/index.html.md` (see
/// `src/markdown.rs`).
pub fn markdown_twin() -> String {
    let HomeProps { title, message } = content();
    format!("# {title}\n\n{message}\n")
}

#[handler]
pub async fn index(req: Request) -> Response {
    // Name the Markdown twin for crawlers. With `serverHead`, the client
    // adds and removes the link as visitors move between pages.
    markdown::advertise(markdown::HOME)?;
    // Typed, so `suprnova generate-types` pairs the page with `HomeProps`.
    let props: HomeProps = content();
    inertia_response!(&req, "Home", props)
}

// TEMPORARY violating examples that bind the `seo` check. Never commit.
#[handler]
pub async fn violation_robots(_req: Request) -> Response {
    Ok(suprnova::HttpResponse::bytes_body("User-agent: *\n", "text/plain; charset=utf-8"))
}

#[handler]
pub async fn violation_sitemap(_req: Request) -> Response {
    Ok(suprnova::HttpResponse::bytes_body(
        "<urlset></urlset>\n",
        "application/xml; charset=utf-8",
    ))
}
