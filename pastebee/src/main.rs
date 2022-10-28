#[macro_use]
extern crate rocket;

mod content;
mod embeds;
mod gateway;

use std::borrow::Cow;
use std::collections::HashMap;
use std::env;

use rocket::fs::{relative, FileServer};
use rocket::request::FromParam;
use rocket::tokio::io;
use rocket::{fs::NamedFile, http::Status};
use rocket_db_pools::{Connection, Database};
use rocket_dyn_templates::Template;

use embeds::EmbedId;
use gateway::auth::AuthState;

#[derive(Database)]
#[database("posts")]
pub struct PostsDbConn(sqlx::SqlitePool);
pub struct Id<'a>(&'a str);

pub type DbResult<T> = std::result::Result<T, rocket::response::Debug<sqlx::Error>>;

impl<'a> FromParam<'a> for Id<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            .then(|| Id(param.into()))
            .ok_or(param)
    }
}

pub fn filepath<'a>(relative: &'a str) -> String {
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.to_str().unwrap();

    current_dir.to_owned() + relative
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(PostsDbConn::init())
        .attach(embeds::stage())
        .attach(gateway::stage())
        .attach(content::stage())
        .mount(
            "/",
            routes![index, favicon, retrieve_generic, retrieve_video],
        )
        .mount("/assets", FileServer::from(filepath("/assets")))
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

#[get("/<id>")]
async fn retrieve_generic<'a>(
    mut db: Connection<PostsDbConn>,
    id: Id<'a>,
) -> Result<Template, Status> {
    let mut exists: HashMap<&'a str, bool> = HashMap::new();
    let mut num_exists = 0;

    let embed = sqlx::query("SELECT id FROM embeds WHERE id=?")
        .bind(&id.0)
        .fetch_one(&mut *db)
        .await;

    match embed {
        Ok(_) => {
            exists.insert("embed", true);
            num_exists += 1;
        }
        Err(_) => {}
    }

    if num_exists == 0 {
        Err(Status::NotFound)
    } else if num_exists == 1 {
        if *exists.get("embed").unwrap_or(&false) {
            embeds::retrieve(db, EmbedId(Cow::Borrowed(id.0))).await
        } else {
            Err(Status::Gone)
        }
    } else {
        Ok(Template::render("multiple", exists))
    }
}

#[get("/watch?<v>")]
async fn retrieve_video(v: String) -> std::io::Result<NamedFile> {
    let filename = v + ".html";
    let path = "/assets/".to_owned() + &filename;
    let path = filepath(&path);

    NamedFile::open(path).await
}
