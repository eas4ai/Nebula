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
