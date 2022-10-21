mod embed_id;
mod hex_color;

use crate::{
    login::auth::{Auth, AuthState},
    DbResult, PostsDbConn, STATIC_PATH,
};
use const_format::concatcp;
use embed_id::EmbedId;
use hex_color::HexColor;
use rocket::fs::NamedFile;
use rocket::tokio::io;
use rocket::{fairing::AdHoc, serde::Serialize};
use rocket::{form::Form, response::Redirect};
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
    site_name: String,
    title: String,
    color: String,
    description: String,
    image: String,
}

impl From<SqliteRow> for TemplateContext {
    fn from(value: SqliteRow) -> Self {
        let site_name = value.try_get(0).unwrap_or(String::new());
        let title = value.try_get(1).unwrap_or(String::new());
        let color = value.try_get(2).unwrap_or(String::new());
        let description = value.try_get(3).unwrap_or(String::new());
        let image = value.try_get(4).unwrap_or(String::new());

        Self {
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
        rocket.mount("/embed", routes![index, retrieve, retrieve_raw, upload])
    })
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: &str;

    if auth.valid {
        path = concatcp!(STATIC_PATH, "/static/embeds/post.html");
    } else {
        path = concatcp!(STATIC_PATH, "/static/embeds/noauth.html");
    }
    NamedFile::open(path).await
}

#[get("/<id>")]
async fn retrieve<'a>(mut db: Connection<PostsDbConn>, id: EmbedId<'a>) -> DbResult<Template> {
    let row =
        sqlx::query("SELECT site_name, title, color, description, image FROM embeds WHERE id=?")
            .bind(&id.0)
            .fetch_one(&mut *db)
            .await?;

    let context: TemplateContext = row.into();

    dbg!(&context);

    Ok(Template::render("embed", context))
}

#[get("/<id>/raw")]
async fn retrieve_raw<'a>(mut db: Connection<PostsDbConn>, id: EmbedId<'a>) -> DbResult<Template> {
    let row =
        sqlx::query("SELECT site_name, title, color, description, image FROM embeds WHERE id=?")
            .bind(&id.0)
            .fetch_one(&mut *db)
            .await?;

    let context: TemplateContext = row.into();

    dbg!(&context);

    Ok(Template::render("embedraw", context))
}

#[post("/", data = "<embed>")]
async fn upload<'a>(
    _auth: Auth,
    mut db: Connection<PostsDbConn>,
    embed: Form<UploadData<'a>>,
) -> DbResult<Template> {
    let id = &embed.id;
    let site_name = &embed.site_name;
    let title = &embed.title;
    let color = &embed.color;
    let description = &embed.description;
    let image = &embed.image;

    sqlx::query("INSERT INTO embeds (id, site_name, title, color, description, image) VALUES (?, ?, ?, ?, ?, ?)")
		.bind(&id.0)
		.bind(site_name)
		.bind(title)
		.bind(color.0)
		.bind(description)
		.bind(image)
		.execute(&mut *db)
		.await?;

    retrieve(db, embed.id.clone()).await
}
