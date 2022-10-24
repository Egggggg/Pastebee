use argon2::{Argon2, PasswordHasher};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    response::Responder,
};
use rocket_dyn_templates::{context, Template};

use super::password::read_password;

#[derive(Debug)]
pub enum AuthError {
    NotAuthed,
}

#[derive(Responder)]
pub enum LoginResponse {
    #[response(status = 500)]
    NoPassword(Template),
    #[response(status = 401)]
    WrongPassword(Template),
    #[response(status = 200)]
    ValidPassword(Template),
    #[response(status = 200)]
    AlreadyAuthed(Template),
}

#[derive(Responder)]
pub enum LogoutResponse {
    #[response(status = 200)]
    LoggedOut(Template),
    #[response(status = 400)]
    NotLoggedIn(Template),
}

// AuthCookie should be used on routes that need auth state, but should still be available to everyone
pub struct AuthState {
    pub valid: bool,
}

// Just a wrapper that responds with a failure on failure to authenticate
// Should be used on routes that should be unavailable to unauthenticated users
pub struct Auth {
    pub valid: bool,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthState {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        let authed = Auth::from_request(request).await;

        if authed.is_success() {
            Outcome::Success(AuthState { valid: true })
        } else {
            Outcome::Success(AuthState { valid: false })
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Auth {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        let cookie = request.cookies().get_private("Authorization");

        if cookie.is_none() {
            return Outcome::Failure((Status { code: 401 }, AuthError::NotAuthed));
        }

        let authed = cookie.unwrap();

        // we can unwrap here cause AuthState always returns Outcome::Success, just with a different inner value
        if authed.value() == "valid" {
            Outcome::Success(Auth { valid: true })
        } else {
            Outcome::Failure((Status { code: 401 }, AuthError::NotAuthed))
        }
    }
}

pub async fn validate_password<'a>(received: &'a str) -> LoginResponse {
    let password = read_password().await;

    if password.is_err() {
        return LoginResponse::NoPassword(Template::render(
            "login",
            context! { message: "someone fucked up" },
        ));
    }

    let password = password.unwrap();

    let salt = password.0;
    let password = password.1;

    let argon2 = Argon2::default();

    let hash = argon2.hash_password(received.as_bytes(), &salt);

    if hash.is_err() {
        return LoginResponse::NoPassword(Template::render("login", context! { message: "how" }));
    }

    let hash = hash.unwrap().to_string();

    if hash == password {
        LoginResponse::ValidPassword(Template::render(
            "login",
            context! { message: "authenticated :D" },
        ))
    } else {
        LoginResponse::WrongPassword(Template::render(
            "login",
            context! { message: "wrong password" },
        ))
    }
}
