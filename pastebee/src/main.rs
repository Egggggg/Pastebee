#[macro_use]
extern crate rocket;

mod embeds;

use const_format::concatcp;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::tokio::io::{self, AsyncReadExt};
use rocket_db_pools::Database;

const STATIC_PATH: &'static str = env!("CARGO_MANIFEST_DIR");

#[derive(Database)]
#[database("posts")]
pub(crate) struct PostsDbConn(sqlx::SqlitePool);

pub(crate) type DbResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
enum AuthError {
    WrongPassword,
    NoPassword,
}

// AuthCookie should be used on routes that need auth state, but should still be available to everyone
pub(crate) struct AuthState(bool);

// Just a wrapper that responds with a failure on failure to authenticate
// Should be used on routes that should be unavailable to unauthenticated users
pub(crate) struct Auth(bool);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthState {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<AuthState, AuthError> {
        let password = read_password().await;

        if password.is_err() {
            return Outcome::Failure((Status { code: 500 }, AuthError::NoPassword));
        }

        let password = password.unwrap();
        let cookie = request.cookies().get_private("password");

        if cookie.is_none() {
            return Outcome::Success(AuthState(false));
        }

        let cookie = cookie.unwrap().value().to_owned();

        if password == cookie {
            Outcome::Success(AuthState(true))
        } else {
            Outcome::Success(AuthState(false))
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Auth {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Auth, AuthError> {
        let authed = AuthState::from_request(request).await;

        if authed.is_failure() {
            Outcome::Failure((Status { code: 401 }, AuthError::WrongPassword))
        } else {
            Outcome::Success(Auth(true))
        }
    }
}

async fn read_password() -> io::Result<String> {
    let mut password = String::new();
    let mut cred_file = NamedFile::open("creds").await?;

    cred_file.read_to_string(&mut password).await?;

    Ok(password)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(PostsDbConn::init())
        .attach(embeds::stage())
        .mount("/", routes![index])
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: &str;

    if auth.0 {
        path = concatcp!(STATIC_PATH, "/static/index.html");
    } else {
        path = concatcp!(STATIC_PATH, "/static/login.html");
    }

    NamedFile::open(path).await
}
