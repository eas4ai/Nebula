//! Search and social metadata: the head every page starts from, the error
//! pages' robots rule, Home's JSON-LD, and `/robots.txt` and `/sitemap.xml`.
//!
//! The head tags come from Suprnova's `Head` API and reach both the first,
//! server-rendered visit and the `head` prop that `serverHead` applies on
//! later visits. Titles stay in the Svelte pages. The canonical link is built
//! from `APP_URL`, never from the request's `Host`.
//!
//! [`RobotsTxt`] and [`Sitemap`] are global middleware placed before the
//! session middleware (see `bootstrap::try_register_http_stack`), as
//! `/llms.txt` is: their responses are publicly cacheable, so they must never
//! carry a cookie. Both answer only `GET` and `HEAD`.

use suprnova::head::{
    ErrorPages, HeadBuilder, OgType, OpenGraph, Schema, TwitterCard, TwitterCardType,
};
use suprnova::{
    FrameworkError, Head, HttpResponse, Middleware, Next, Request, Response, async_trait,
};

use crate::controllers::{app_name, app_url};
use crate::markdown::{PAGES, SUMMARY};

/// The image social sites show for a page: the kit's largest icon. Replace
/// it with a 1200x630 card and switch the X card to `SummaryLargeImage`.
pub const SOCIAL_IMAGE: &str = "/android-chrome-512x512.png";

/// `path` as an absolute URL on `APP_URL`.
fn absolute(base_url: &str, path: &str) -> String {
    format!("{}{path}", base_url.trim_end_matches('/'))
}

/// The Open Graph tags every page shares.
fn site_og() -> OpenGraph {
    OpenGraph::new().kind(OgType::Website).site_name(app_name())
}

/// Register the head defaults every page starts from and the error pages'
/// robots rule. Each call replaces what an earlier one registered, so
/// registering the HTTP stack twice adds no tag twice.
pub fn register() {
    let image = absolute(&app_url(), SOCIAL_IMAGE);
    Head::defaults(|_| {
        HeadBuilder::new()
            .canonical()
            .canonical_keep_scheme()
            .description(SUMMARY)
            .og(site_og())
            .og_image(image)
            .twitter(TwitterCard::new().card(TwitterCardType::Summary))
    });
    Head::errors(|_| ErrorPages::default().defaults(|head| head.robots("noindex")));
}

/// Home's own head: its description and `og:title` from the text it shows,
/// and JSON-LD describing the site.
pub fn describe_home(title: &str, message: &str) -> Result<(), FrameworkError> {
    let base = app_url();
    let home = absolute(&base, "/");
    let website = Schema::of("WebSite")
        .set("name", app_name())
        .set("url", home.clone());
    let organization = Schema::of("Organization")
        .set("name", app_name())
        .set("url", home)
        .set("logo", absolute(&base, SOCIAL_IMAGE));
    Head::try_update(|head| {
        head.description(message)
            .og(site_og().title(title))
            .schema(website)
            .schema(organization)
    })
}

/// The `/robots.txt` body: every crawler may fetch every path, and the
/// sitemap is on `APP_URL`. Pages that must stay out of search results say
/// so with a robots meta tag, which a crawler only reads if it may fetch
/// the page.
pub fn robots_txt(base_url: &str) -> String {
    format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}\n",
        absolute(base_url, "/sitemap.xml")
    )
}

/// The `/sitemap.xml` body: one `<url>` per public page in [`PAGES`].
pub fn sitemap_xml(base_url: &str) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in PAGES {
        out.push_str(&format!(
            "  <url><loc>{}</loc></url>\n",
            xml_escape(&absolute(base_url, page.href))
        ));
    }
    out.push_str("</urlset>\n");
    out
}

/// Escape the five characters XML treats specially.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// A publicly cacheable answer for an hour, as `/llms.txt` sends.
fn public_hour(body: String, content_type: &str) -> HttpResponse {
    HttpResponse::bytes_body(body, content_type).header("Cache-Control", "public, max-age=3600")
}

/// Whether `request` is a `GET` or `HEAD` of `path`.
fn reads(request: &Request, path: &str) -> bool {
    request.path() == path && matches!(request.method().as_str(), "GET" | "HEAD")
}

/// Answers `GET` and `HEAD /robots.txt`; every other request passes through.
pub struct RobotsTxt;

#[async_trait]
impl Middleware for RobotsTxt {
    async fn handle(&self, request: Request, next: Next) -> Response {
        if !reads(&request, "/robots.txt") {
            return next(request).await;
        }
        // `Err` is how a middleware answers without calling the router.
        Err(public_hour(
            robots_txt(&app_url()),
            "text/plain; charset=utf-8",
        ))
    }
}

/// Answers `GET` and `HEAD /sitemap.xml`; every other request passes through.
pub struct Sitemap;

#[async_trait]
impl Middleware for Sitemap {
    async fn handle(&self, request: Request, next: Next) -> Response {
        if !reads(&request, "/sitemap.xml") {
            return next(request).await;
        }
        Err(public_hour(
            sitemap_xml(&app_url()),
            "application/xml; charset=utf-8",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_txt_names_the_sitemap_once_slashed() {
        assert_eq!(
            robots_txt("https://example.test/"),
            "User-agent: *\nAllow: /\n\nSitemap: https://example.test/sitemap.xml\n"
        );
    }

    #[test]
    fn sitemap_lists_each_public_page_escaped() {
        let body = sitemap_xml("https://example.test/a&b/");
        assert!(
            body.contains("<loc>https://example.test/a&amp;b/</loc>"),
            "{body}"
        );
        assert_eq!(body.matches("<url>").count(), PAGES.len());
    }
}
