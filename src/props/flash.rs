//! The flash data a page shows once.
//!
//! A handler that finishes an action flashes `success` or `error` copy with
//! `redirect!(...).with(...)`. The next page response carries it as
//! `page.flash` and removes it, so `Layout.svelte` shows it once.

use suprnova::InertiaProps;

/// What `page.flash` holds. The marker makes `suprnova generate-types` name
/// this struct as Inertia's `flashDataType`, which types `page.flash` on
/// every page. Handlers never build it: each flashes its one key.
#[derive(InertiaProps)]
#[inertia_props(flash)]
pub struct Flash {
    /// Copy for an action that worked, such as a password reset.
    pub success: Option<String>,
    /// Copy for an action that failed.
    pub error: Option<String>,
}
