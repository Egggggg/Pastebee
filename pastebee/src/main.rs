#[macro_use]
extern crate rocket;

mod embeds;
mod login;

use std::env;

use rocket::fs::NamedFile;
use rocket::tokio::io;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

use login::auth::AuthState;

#[derive(Database)]
#[database("posts")]
pub struct PostsDbConn(sqlx::SqlitePool);

pub type DbResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

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
        .attach(login::stage())
        .mount("/", routes![index])
        .attach(Template::fairing())
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
