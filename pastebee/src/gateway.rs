pub mod auth;
mod password;

use auth::AuthState;
use rocket::{
    fairing::AdHoc,
    form::Form,
    fs::NamedFile,
    http::{Cookie, CookieJar},
    tokio::io,
};
use rocket_dyn_templates::{context, Template};

use crate::filepath;
use auth::{validate_password, LoginResponse, LogoutResponse};

#[derive(FromForm)]
struct Login {
    password: String,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Login stage", |rocket| async {
        rocket
            .mount("/login", routes![index, login])
            .mount("/logout", routes![index, logout])
    })
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: String;

    if auth.valid {
        path = filepath("/static/auth/logout.html");
    } else {
        path = filepath("/static/auth/login.html");
    }

    NamedFile::open(path).await
}

#[post("/", data = "<login>")]
async fn login<'a>(auth: AuthState, cookies: &CookieJar<'a>, login: Form<Login>) -> LoginResponse {
    if auth.valid {
        return LoginResponse::AlreadyAuthed(Template::render(
            "login",
            context! { message: "already logged in" },
        ));
    }

    let valid = validate_password(&login.password).await;

    match valid {
        LoginResponse::ValidPassword(_) => {
            cookies.add_private(Cookie::new("Authorization", "valid"));
            valid
        }
        _ => valid,
    }
}

#[post("/")]
async fn logout<'a>(auth: AuthState, cookies: &CookieJar<'a>) -> LogoutResponse {
    if auth.valid {
        cookies.remove_private(Cookie::named("Authorization"));

        LogoutResponse::LoggedOut(Template::render(
            "logout",
            context! { message: "successfully logged out"},
        ))
    } else {
        LogoutResponse::NotLoggedIn(Template::render(
            "logout",
            context! { message: "not even logged in" },
        ))
    }
}
