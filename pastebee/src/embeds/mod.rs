mod embed_id;
mod hex_color;

use crate::{Auth, DbResult, PostsDbConn};
use embed_id::EmbedId;
use hex_color::HexColor;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::tokio::fs::File;
use rocket::tokio::io;
use rocket_db_pools::Connection;

#[derive(FromForm)]
struct UploadData<'a> {
    id: EmbedId<'a>,
    site_name: String,
    title: String,
    color: HexColor<'a>,
    description: String,
    image: String,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Embeds stage", |rocket| async {
        rocket.mount("/embed", routes![index, retrieve, upload])
    })
}

#[get("/")]
async fn index() -> io::Result<NamedFile> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/embeds/index.html");
    NamedFile::open(path).await
}

#[get("/<id>")]
async fn retrieve<'a>(id: EmbedId<'a>) -> Option<File> {
    File::open(id.file_path()).await.ok()
}

#[post("/", data = "<embed>")]
async fn upload<'a>(
    auth: Auth,
    mut db: Connection<PostsDbConn>,
    embed: Form<UploadData<'a>>,
) -> DbResult<String> {
    let id = &embed.id;
    let site_name = &embed.site_name;
    let title = &embed.title;
    let color = &embed.color;
    let description = &embed.description;
    let image = &embed.image;

    sqlx::query("INSERT INTO embeds (id, description, title, site_name, color, image) VALUES (?, ?, ?, ?, ?, ?)")
		.bind(&id.0)
		.bind(description)
		.bind(title)
		.bind(site_name)
		.bind(color.0)
		.bind(image)
		.execute(&mut *db)
		.await?;

    Ok(uri!(retrieve(id)).to_string())
}
