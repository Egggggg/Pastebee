mod embed_id;
mod hex_color;

use crate::{
    filepath,
    gateway::auth::{Auth, AuthState},
    PostsDbConn,
};
use embed_id::EmbedId;
use hex_color::HexColor;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::tokio::io;
use rocket::{fairing::AdHoc, serde::Serialize};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use sqlx::{sqlite::SqliteRow, Row};

#[derive(FromForm)]
struct UploadData<'a> {
    id: EmbedId<'a>,
    site_name: String,
    title: String,
    color: HexColor<'a>,
    description: String,
    image: String,
}

#[derive(Serialize, Debug)]
struct TemplateContext {
    id: String,
    site_name: String,
    title: String,
    color: String,
    description: String,
    image: String,
}

impl From<SqliteRow> for TemplateContext {
    fn from(value: SqliteRow) -> Self {
        let id = value.try_get(0).unwrap_or(String::new());
        let site_name = value.try_get(1).unwrap_or(String::new());
        let title = value.try_get(2).unwrap_or(String::new());
        let color = value.try_get(3).unwrap_or(String::new());
        let description = value.try_get(4).unwrap_or(String::new());
        let image = value.try_get(5).unwrap_or(String::new());

        Self {
            id,
            site_name,
            title,
            color,
            description,
            image,
        }
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Embeds stage", |rocket| async {
        rocket
            .mount("/embed", routes![index, retrieve, retrieve_raw, upload])
            .register("/embed", catchers![invalid])
    })
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: String;

    if auth.valid {
        path = filepath("/static/embeds/post.html");
    } else {
        path = filepath("/static/embeds/noauth.html");
    }
    NamedFile::open(path).await
}

#[get("/<id>")]
async fn retrieve<'a>(
    mut db: Connection<PostsDbConn>,
    id: EmbedId<'a>,
) -> Result<Template, Status> {
    let row = sqlx::query(
        "SELECT id, site_name, title, color, description, image FROM embeds WHERE id=?",
    )
    .bind(&id.0)
    .fetch_one(&mut *db)
    .await;

    if row.is_err() {
        return Err(Status::NotFound);
    }

    let row = row.unwrap();
    let context: TemplateContext = row.into();

    Ok(Template::render("embed", context))
}

#[get("/<id>/raw")]
async fn retrieve_raw<'a>(
    mut db: Connection<PostsDbConn>,
    id: EmbedId<'a>,
) -> Result<Template, Status> {
    let row = sqlx::query(
        "SELECT id, site_name, title, color, description, image FROM embeds WHERE id=?",
    )
    .bind(&id.0)
    .fetch_one(&mut *db)
    .await;

    if row.is_err() {
        return Err(Status::NotFound);
    }

    let row = row.unwrap();
    let context: TemplateContext = row.into();

    dbg!(&context);

    Ok(Template::render("embedraw", context))
}

#[post("/", data = "<embed>")]
async fn upload<'a>(
    _auth: Auth,
    mut db: Connection<PostsDbConn>,
    embed: Form<UploadData<'a>>,
) -> Result<Template, Status> {
    let id = &embed.id;
    let site_name = &embed.site_name;
    let title = &embed.title;
    let color = &embed.color;
    let description = &embed.description;
    let image = &embed.image;

    let query = sqlx::query("INSERT INTO embeds (id, site_name, title, color, description, image) VALUES (?, ?, ?, ?, ?, ?)")
		.bind(&id.0)
		.bind(site_name)
		.bind(title)
		.bind(color.0)
		.bind(description)
		.bind(image)
		.execute(&mut *db)
		.await;

    if query.is_err() {
        Err(Status::Conflict)
    } else {
        retrieve(db, embed.id.clone()).await
    }
}
