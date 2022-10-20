#[macro_use]
extern crate rocket;

mod embeds;

use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::tokio::io::{self, AsyncReadExt};
use rocket_db_pools::Database;

#[derive(Database)]
#[database("posts")]
pub(crate) struct PostsDbConn(sqlx::SqlitePool);

pub(crate) type DbResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
enum AuthError {
    WrongPassword,
    NoPassword,
}

pub(crate) struct AuthLogin(bool);
pub(crate) struct Auth(bool);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthLogin {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<AuthLogin, AuthError> {
        let password = read_password().await;

        if password.is_err() {
            return Outcome::Failure((Status { code: 500 }, AuthError::NoPassword));
        }

        let password = password.unwrap();

        let received = request
            .headers()
            .get_one("Authorization")
            .unwrap_or("")
            .to_owned();

        if password == received {
            Outcome::Success(AuthLogin(true))
        } else {
            Outcome::Failure((Status { code: 401 }, AuthError::WrongPassword))
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Auth {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Auth, AuthError> {
        let password = read_password().await;

        if password.is_err() {
            return Outcome::Success(Auth(false));
        }

        let password = password.unwrap();
        let cookie = request.cookies().get_private("password");

        if cookie.is_none() {
            return Outcome::Success(Auth(false));
        }

        let cookie = cookie.unwrap().value().to_owned();

        if password == cookie {
            Outcome::Success(Auth(true))
        } else {
            Outcome::Success(Auth(false))
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
async fn index(auth: Auth) -> io::Result<NamedFile> {
    let path: &str;

    if !auth.0 {
        path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/login.html");
    } else {
        path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html");
    }

    NamedFile::open(path).await
}
