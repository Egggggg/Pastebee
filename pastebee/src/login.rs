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
use auth::{validate_password, LoginResponse};
use password::Password;

#[derive(FromForm)]
struct Login {
    password: Password,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Login stage", |rocket| async {
        rocket.mount("/login", routes![index, login])
    })
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: String;

    if auth.valid {
        path = filepath("/static/index.html");
    } else {
        path = filepath("/static/login.html");
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
