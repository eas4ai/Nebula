//! Markdown twins of the public pages, and the `/llms.txt` index.
//!
//! An Inertia page is an empty shell until JavaScript runs, so crawlers that
//! do not run it (GPTBot, ClaudeBot, PerplexityBot, `curl`) see nothing. Each
//! page in [`PAGES`] gets a Markdown copy, its twin, at its path plus `.md`,
//! served by `suprnova-markdown`'s `MarkdownMiddleware`. `/llms.txt` lists
//! the twins. Both come from [`PAGES`], so they cannot disagree.
//!
//! Both middlewares run before the session middleware (see
//! `bootstrap::try_register_http_stack`). Their responses are publicly
//! cacheable, so they must never start a session or carry a cookie.
//!
//! The four rules for every page you add here and every source or
//! middleware you write:
//!
//! 1. Public pages only. Return Markdown only for pages anyone may read,
//!    decided on each request from current data. These middlewares run
//!    before the router's sign-in and permission checks.
//! 2. Caching. Serve twins of pages that can stop being public
//!    (unpublished, deleted, made private) with
//!    `MarkdownMiddleware::max_age(0)`.
//! 3. Methods and errors. Answer only `GET` and `HEAD`, and answer a failed
//!    lookup with a 500 that hides the error text. The plugin does both for
//!    twins; [`LlmsTxt`] checks the method itself.
//! 4. No cookies. Keep both middlewares before the session middleware.
//!
//! A source that reads a database must also limit its own time: twins run
//! outside the request timeout.

use suprnova::head::LinkTag;
use suprnova::{
    FrameworkError, Head, HttpResponse, Middleware, Next, Request, Response, async_trait,
};
use suprnova_markdown::MarkdownSource;

use crate::controllers::{app_url, home};

/// The summary line under the title in `/llms.txt`. Describe your app here.
pub const SUMMARY: &str = "A Suprnova starter kit: sign-up, sign-in, email verification, password reset and profile management, on Rust, Inertia and Svelte.";

/// The site root's page path. Its twin is `/index.html.md`, as the llms.txt
/// proposal names it.
pub const HOME: &str = "/index.html";

/// A public page with a Markdown twin.
pub struct PublicPage {
    /// The page's path as its twin's address names it, without `.md`.
    pub path: &'static str,
    /// The page's name in `/llms.txt`.
    pub title: &'static str,
    /// Builds the page's Markdown from the data the page renders.
    pub markdown: fn() -> String,
}

/// Every page with a twin. Add an entry for each public prose page.
pub const PAGES: &[PublicPage] = &[PublicPage {
    path: HOME,
    title: "Home",
    markdown: home::markdown_twin,
}];

/// The twin's address for a page path from [`PAGES`].
pub fn twin_path(page_path: &str) -> String {
    format!("{page_path}.md")
}

/// The `<link rel="alternate" type="text/markdown">` naming a page's twin,
/// or `None` when [`PAGES`] does not list the page.
pub fn alternate(page_path: &str) -> Option<LinkTag> {
    PAGES
        .iter()
        .any(|page| page.path == page_path)
        .then(|| LinkTag::new("alternate", twin_path(page_path)).attribute("type", "text/markdown"))
}

/// Names the page's twin in its head, so crawlers can find it. Call it from
/// the handler of each page in [`PAGES`]. A path [`PAGES`] does not list
/// gets no link, so a page never names a twin that does not exist.
pub fn advertise(page_path: &str) -> Result<(), FrameworkError> {
    if let Some(link) = alternate(page_path) {
        Head::try_update(|head| head.link_tag(link))?;
    }
    Ok(())
}

/// Serves the twins of [`PAGES`]. Any other path is `Ok(None)`, and the
/// router answers it.
pub struct PublicPages;

#[async_trait]
impl MarkdownSource for PublicPages {
    async fn markdown(&self, path: &str) -> Result<Option<String>, FrameworkError> {
        Ok(PAGES
            .iter()
            .find(|page| page.path == path)
            .map(|page| (page.markdown)()))
    }
}

/// The `/llms.txt` body (llmstxt.org): the app's name, [`SUMMARY`], and an
/// absolute link to each twin, because readers fetch the links out of
/// context.
pub fn llms_txt(app_name: &str, base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let mut out = format!("# {app_name}\n\n> {SUMMARY}\n\n## Pages\n\n");
    for page in PAGES {
        out.push_str(&format!(
            "- [{}]({base}{})\n",
            page.title,
            twin_path(page.path)
        ));
    }
    out
}

/// Answers `GET` and `HEAD /llms.txt`; every other request passes through.
///
/// A middleware, not a route: global middleware runs for every route, and
/// the CSRF middleware sets an `XSRF-TOKEN` cookie on each response. Served
/// from here, before the session, the publicly cacheable index carries no
/// cookie.
pub struct LlmsTxt;

#[async_trait]
impl Middleware for LlmsTxt {
    async fn handle(&self, request: Request, next: Next) -> Response {
        if request.path() != "/llms.txt" || !matches!(request.method().as_str(), "GET" | "HEAD") {
            return next(request).await;
        }
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Nebula".to_string());
        // `Err` is how a middleware answers without calling the router.
        Err(
            HttpResponse::bytes_body(llms_txt(&app_name, &app_url()), "text/plain; charset=utf-8")
                .header("Cache-Control", "public, max-age=3600"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_twin_is_built_from_the_home_props() {
        let props = home::content();
        assert_eq!(
            home::markdown_twin(),
            format!("# {}\n\n{}\n", props.title, props.message)
        );
    }

    #[test]
    fn the_site_root_twin_follows_the_llms_txt_convention() {
        assert_eq!(twin_path(HOME), "/index.html.md");
    }

    #[tokio::test]
    async fn serves_listed_pages_only() {
        let twin = PublicPages.markdown("/index.html").await.expect("lookup");
        assert_eq!(twin, Some(home::markdown_twin()));
        for path in ["/index", "/", "/login", "/dashboard", "/index.html/"] {
            let twin = PublicPages.markdown(path).await.expect("lookup");
            assert_eq!(twin, None, "{path} has no twin");
        }
    }

    #[test]
    fn only_listed_pages_get_an_alternate_link() {
        assert_eq!(
            alternate(HOME),
            Some(LinkTag::new("alternate", "/index.html.md").attribute("type", "text/markdown"))
        );
        for path in ["/", "/login", "/index.html.md"] {
            assert_eq!(alternate(path), None, "{path} is not listed");
        }
    }

    #[test]
    fn llms_txt_links_every_twin_absolutely() {
        // A trailing slash on APP_URL must not double the slash.
        let body = llms_txt("Nebula", "https://example.test/");
        assert_eq!(
            body,
            format!(
                "# Nebula\n\n> {SUMMARY}\n\n## Pages\n\n- [Home](https://example.test/index.html.md)\n"
            )
        );
    }
}
