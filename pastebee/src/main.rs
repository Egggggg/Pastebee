#[macro_use]
extern crate rocket;

mod embeds;
mod login;

use const_format::concatcp;
use rocket::fs::NamedFile;
use rocket::tokio::io;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

use login::auth::AuthState;

const STATIC_PATH: &'static str = env!("CARGO_MANIFEST_DIR");

#[derive(Database)]
#[database("posts")]
pub struct PostsDbConn(sqlx::SqlitePool);

pub type DbResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

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
    let path: &str;

    if auth.valid {
        path = concatcp!(STATIC_PATH, "/static/index.html");
    } else {
        path = concatcp!(STATIC_PATH, "/static/login.html");
    }

    NamedFile::open(path).await
}
