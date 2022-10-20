#[macro_use]
extern crate rocket;

mod embed_id;

use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::http::uri::Absolute;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncReadExt;

use embed_id::EmbedId;

const ID_LENGTH: usize = 3;
const HOST: Absolute<'static> = uri!("http://localhost:8000");

#[derive(FromForm)]
struct UploadData<'a> {
    desc: String,
    title: String,
    color: String,
    image: String,
    id: EmbedId<'a>,
}

#[derive(Debug)]
enum AuthError {
    WrongPassword,
    NoPassword,
}

struct Auth(bool);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Auth {
    type Error = AuthError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Auth, AuthError> {
        let mut password = String::new();
        let cred_file = NamedFile::open("creds").await;

        if cred_file.is_err() {
            return Outcome::Failure((Status { code: 500 }, AuthError::NoPassword));
        }

        cred_file
            .unwrap()
            .read_to_string(&mut password)
            .await
            .unwrap();

        let received = request
            .headers()
            .get_one("password")
            .unwrap_or("")
            .to_owned();

        if password == received {
            Outcome::Success(Auth(true))
        } else {
            Outcome::Failure((Status { code: 403 }, AuthError::WrongPassword))
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, retrieve, upload])
}

#[get("/")]
fn index() -> &'static str {
    "
	USAGE

		POST /

			accepts raw data in the body and responds with
			the URL to a page containing the body's content

		GET /<id>

			retrieves the content for the paste with id `<id>`
	"
}

#[get("/<id>")]
async fn retrieve<'a>(id: EmbedId<'a>) -> Option<File> {
    File::open(id.file_path()).await.ok()
}

#[post("/", data = "<paste>")]
async fn upload(paste: Form<UploadData>, auth: Auth) -> Result<String, UploadError> {
    if paste.id.is_empty() || !paste.id.chars().all(|c| c.is_ascii_alphanumeric()) {}
}
