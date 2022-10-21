pub mod auth;
mod password;

use auth::AuthState;
use rocket::{
    fairing::AdHoc,
    form::Form,
    http::{Cookie, CookieJar, Status},
};

use auth::validate_password;
use password::Password;

#[derive(FromForm)]
struct Login {
    password: Password,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Login stage", |rocket| async {
        rocket.mount("/login", routes![login])
    })
}

#[post("/", data = "<login>")]
async fn login<'a>(
    auth: AuthState,
    cookies: &CookieJar<'a>,
    login: Form<Login>,
) -> (Status, &'static str) {
    if auth.valid {
        return (Status { code: 202 }, "already logged in");
    }

    let valid = validate_password(login.password).await;

	if valid.is_failure() {
		return (Status { code: })
	}

    if valid {
        cookies.add_private(Cookie::new("Authorization", "valid"));
        (Status { code: 202 }, "successfully logged in")
    } else {
        (Status { code: 401 }, "login failed")
    }
}
