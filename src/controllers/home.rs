use suprnova::{InertiaProps, Request, Response, handler, inertia_response};

#[derive(InertiaProps)]
pub struct HomeProps {
    pub title: String,
    pub message: String,
}

#[handler]
pub async fn index(req: Request) -> Response {
    inertia_response!(
        &req,
        "Home",
        HomeProps {
            title: "Welcome to Suprnova!".to_string(),
            message: "Your Inertia app is ready.".to_string(),
        }
    )
}

// TEMPORARY violating examples that bind the `twins` checks. Never commit.
static VIOLATION_HITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[handler]
pub async fn violation_twin(req: Request) -> Response {
    if req.method().as_str() == "HEAD" {
        return Ok(suprnova::HttpResponse::text("").status(404));
    }
    let n = VIOLATION_HITS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(
        suprnova::HttpResponse::bytes_body(format!("# Home {n}\n"), "text/markdown; charset=utf-8")
            .header("X-Robots-Tag", "noindex")
            .header("Cache-Control", "public, max-age=3600"),
    )
}

#[handler]
pub async fn violation_markdown(_req: Request) -> Response {
    Ok(suprnova::HttpResponse::bytes_body("# Not public\n", "text/markdown; charset=utf-8"))
}

pub struct ViolationLlms;

#[suprnova::async_trait]
impl suprnova::Middleware for ViolationLlms {
    async fn handle(&self, request: Request, next: suprnova::Next) -> Response {
        if request.path() != "/llms.txt" || request.method().as_str() != "GET" {
            return next(request).await;
        }
        Err(suprnova::HttpResponse::bytes_body(
            format!("- [Home]({}/index.html.md)\n", super::app_url()),
            "text/plain; charset=utf-8",
        )
        .header("X-Content-Type-Options", "sniff"))
    }
}
