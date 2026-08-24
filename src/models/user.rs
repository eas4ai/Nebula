//! User model.
//!
//! Defined with the `#[suprnova::model]` macro: the struct below *is* the
//! Eloquent model. The macro emits the SeaORM `Entity` / `Column` /
//! `ActiveModel` in an inner `user` module and gives `User` the query surface
//! (`User::query()`, `User::find()`, the `Model::create` mass-assignment entry
//! point, `save`, timestamps). `Authenticatable` is implemented on the struct
//! so the auth stack (session middleware, user providers, `Auth::user()`) can
//! resolve users without touching SeaORM directly.

use std::any::Any;

use chrono::{DateTime, FixedOffset, Utc};
use suprnova::{
    Authenticatable, CanResetPassword, Cast, FrameworkError, MustVerifyEmail, attrs, hashing, model,
};

struct NativeUtcDateTime;

impl Cast for NativeUtcDateTime {
    type Runtime = DateTime<Utc>;
    type Storage = DateTime<FixedOffset>;

    fn to_storage(value: &Self::Runtime) -> Result<Self::Storage, FrameworkError> {
        Ok(value.fixed_offset())
    }

    fn from_storage(stored: &Self::Storage) -> Result<Self::Runtime, FrameworkError> {
        Ok(stored.with_timezone(&Utc))
    }
}

struct OptionalNativeUtcDateTime;

impl Cast for OptionalNativeUtcDateTime {
    type Runtime = Option<DateTime<Utc>>;
    type Storage = Option<DateTime<FixedOffset>>;

    fn to_storage(value: &Self::Runtime) -> Result<Self::Storage, FrameworkError> {
        Ok(value.as_ref().map(DateTime::fixed_offset))
    }

    fn from_storage(stored: &Self::Storage) -> Result<Self::Runtime, FrameworkError> {
        Ok(stored.as_ref().map(|value| value.with_timezone(&Utc)))
    }
}

#[model(
    table = "app_users",
    fillable = ["name", "email", "password_hash"],
    hidden = ["password_hash", "remember_token", "locked_at", "auth_epoch", "session_version"],
    casts = {
        created_at = NativeUtcDateTime,
        updated_at = NativeUtcDateTime,
        email_verified_at = OptionalNativeUtcDateTime,
        locked_at = OptionalNativeUtcDateTime
    },
    timestamps,
)]
pub struct User {
    pub id: i64,
    pub name: Option<String>,
    pub email: String,
    pub password_hash: Option<String>,
    pub remember_token: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
    pub auth_epoch: i64,
    pub session_version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Re-export the SeaORM types the macro emits in the inner `user` module so
// call sites referencing `crate::models::user::{Entity, Column, ActiveModel}`
// keep resolving.
pub use user::{ActiveModel, Column, Entity};

impl User {
    /// Find a user by their email address.
    pub async fn find_by_email(email: &str) -> Result<Option<Self>, FrameworkError> {
        <Self as suprnova::eloquent::Model>::query()
            .filter("email", email)
            .first()
            .await
    }

    /// Verify a plaintext password against this user's stored hash.
    pub fn verify_password(&self, password: &str) -> Result<bool, FrameworkError> {
        match self
            .password_hash
            .as_deref()
            .filter(|hash| !hash.is_empty())
        {
            Some(hash) => hashing::verify(password, hash),
            None => Ok(false),
        }
    }

    /// Create a new user, hashing the password before insert. Values are
    /// mass-assigned through the model's `fillable` set.
    pub async fn create(
        name: impl Into<String>,
        email: impl Into<String>,
        password: &str,
    ) -> Result<Self, FrameworkError> {
        let name: String = name.into();
        let email: String = email.into();
        let hashed = hashing::hash(password)?;
        <Self as suprnova::eloquent::Model>::create(attrs! {
            name: Some(name),
            email: email,
            password_hash: Some(hashed),
        })
        .await
    }

    /// Persist only the profile-owned user columns, leaving authentication
    /// state (`password_hash`, epochs, lockout, and remember credentials)
    /// untouched even if this model snapshot is stale.
    pub async fn update_profile(
        self,
        name: String,
        email: String,
        email_verified_at: Option<DateTime<Utc>>,
    ) -> Result<Self, FrameworkError> {
        suprnova::unguarded(|| {
            <Self as suprnova::eloquent::Model>::update(
                self,
                attrs! {
                    name: Some(name),
                    email: email,
                    email_verified_at: email_verified_at,
                },
            )
        })
        .await
    }

    /// Persist only the password hash without writing stale epoch or lockout state.
    pub async fn update_password_hash(self, hash: String) -> Result<Self, FrameworkError> {
        <Self as suprnova::eloquent::Model>::update(self, attrs! { password_hash: Some(hash) })
            .await
    }

    /// Persist only the legacy framework remember-token column.
    pub async fn update_remember_token(&self, token: Option<String>) -> Result<(), FrameworkError> {
        let user = self.clone();
        suprnova::unguarded(|| {
            <Self as suprnova::eloquent::Model>::update(user, attrs! { remember_token: token })
        })
        .await
        .map(|_| ())
    }
}

impl Authenticatable for User {
    fn get_auth_identifier(&self) -> String {
        self.id.to_string()
    }

    fn auth_identifier_name(&self) -> &'static str {
        "id"
    }

    /// Expose the stored hash so the built-in `EloquentUserProvider`
    /// can validate credentials - without this override the trait
    /// default returns `None` and `Auth::attempt` rejects EVERY
    /// password, so `POST /login` can never succeed.
    fn get_auth_password(&self) -> Option<&str> {
        self.password_hash
            .as_deref()
            .filter(|hash| !hash.is_empty())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_arc_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn Any + Send + Sync> {
        self
    }
}

// The email-verification flow reads the address + verification timestamp
// through this trait and writes the timestamp back on consume. Implementing
// it (alongside `CanResetPassword` below) is what lets the
// `EloquentUserProvider<User>` registered in `bootstrap::register()` drive
// `EmailVerification::resend` / `verify` against this model.
impl MustVerifyEmail for User {
    fn email(&self) -> &str {
        &self.email
    }

    fn email_verified_at(&self) -> Option<DateTime<Utc>> {
        self.email_verified_at
    }

    fn set_email_verified_at(&mut self, v: Option<DateTime<Utc>>) {
        self.email_verified_at = v;
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref().filter(|name| !name.is_empty())
    }
}

// The password-reset flow addresses its mail through `email_for_reset()` and
// persists the rotated (already-hashed) password through `set_password_hash()`.
impl CanResetPassword for User {
    fn email_for_reset(&self) -> &str {
        &self.email
    }

    fn set_password_hash(&mut self, hash: &str) {
        self.password_hash = Some(hash.to_string());
    }
}
