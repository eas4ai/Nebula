//! The props every page receives: `auth`, the signed-in user or none.
//!
//! The HTTP stack shares `auth` with `App::inertia_share_lazy` (see
//! `bootstrap::try_register_http_stack`), so [`Auth::current`] runs for each
//! response that sends it, with that request's session. `AppLayout.svelte`
//! reads `page.props.auth.user` to render the user menu and the Dashboard
//! link, or the sign-in links when it is `null`.

use suprnova::{FrameworkError, InertiaProps};

use crate::models::user::User;

/// The shared props, as `suprnova generate-types` adds them to the
/// generated `SharedProps` beside the framework's own `root`. The marker
/// only types them; the share in `bootstrap.rs` sends them.
#[derive(InertiaProps)]
#[inertia_props(shared)]
pub struct SharedData {
    pub auth: Auth,
}

/// Who is signed in.
#[derive(InertiaProps)]
pub struct Auth {
    /// The signed-in user, or `None` for a guest.
    pub user: Option<AuthUser>,
}

impl Auth {
    /// The signed-in user of the request being answered, or none.
    pub async fn current() -> Result<Self, FrameworkError> {
        let user = suprnova::Auth::user_as::<User>().await?;
        Ok(Self {
            user: user.map(AuthUser::from),
        })
    }
}

/// A user as the pages show them: never the password hash or a token.
#[derive(InertiaProps)]
pub struct AuthUser {
    pub id: i64,
    /// The user's name, or `"Account"` when they have none, so the user
    /// menu always has a label.
    pub name: String,
    pub email: String,
}

impl From<User> for AuthUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user
                .name
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "Account".to_string()),
            email: user.email,
        }
    }
}
