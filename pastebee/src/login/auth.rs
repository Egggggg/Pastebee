use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};

use super::password::{read_password, Password};

#[derive(Debug)]
pub enum AuthError {
    WrongPassword,
    NoPassword,
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
        let cookie = request.cookies().get_private("password");

        if cookie.is_none() {
            return Outcome::Success(AuthState { valid: false });
        }

        if cookie.unwrap().value() == "valid" {
            Outcome::Success(AuthState { valid: true })
        } else {
            Outcome::Success(AuthState { valid: false })
        }

        /*
        let password = read_password().await;

        if password.is_err() {
            return Outcome::Failure((Status { code: 500 }, AuthError::NoPassword));
        }

        let password = password.unwrap();
        let cookie = request.cookies().get_private("password");

        if cookie.is_none() {
            return Outcome::Success(AuthState { good: false });
        }

        let cookie = cookie.unwrap().value().to_owned();

        if password == cookie {
            Outcome::Success(AuthState { good: true })
        } else {
            Outcome::Success(AuthState { good: false })
        }
        */
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Auth {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        let authed = AuthState::from_request(request).await;

        if authed.is_success() {
            Outcome::Success(Auth { valid: true })
        } else {
            Outcome::Failure((Status { code: 401 }, AuthError::WrongPassword))
        }
    }
}

pub async fn validate_password(received: Password) -> Outcome<bool, AuthError> {
    let password = read_password().await;

    if password.is_err() {
        return Outcome::Failure((Status { code: 500 }, AuthError::NoPassword));
    }

    let password = password.unwrap();

    let mut hasher = DefaultHasher::new();
    received.hash(&mut hasher);
    let hash = hasher.finish().to_string();

    if hash == password {
        Outcome::Success(true)
    } else {
        Outcome::Success(false)
    }
}
