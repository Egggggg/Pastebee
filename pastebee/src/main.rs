#[macro_use]
extern crate rocket;

mod embeds;
mod gateway;

use std::env;

use rocket::fs::NamedFile;
use rocket::tokio::io;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

use gateway::auth::AuthState;

#[derive(Database)]
#[database("posts")]
pub struct PostsDbConn(sqlx::SqlitePool);

pub type DbResult<T> = std::result::Result<T, rocket::response::Debug<sqlx::Error>>;

pub fn filepath<'a>(relative: &'a str) -> String {
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.to_str().unwrap();

    format!("{}{}", current_dir, relative)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(PostsDbConn::init())
        .attach(embeds::stage())
        .attach(gateway::stage())
        .mount("/", routes![index, favicon])
        .attach(Template::fairing())
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: String;

    if auth.valid {
        path = filepath("/static/index.html");
    } else {
        path = filepath("/static/auth/login.html");
    }

    NamedFile::open(path).await
}

#[get("/favicon.ico")]
async fn favicon() -> std::io::Result<NamedFile> {
    NamedFile::open(filepath("/favicon.ico")).await
}
