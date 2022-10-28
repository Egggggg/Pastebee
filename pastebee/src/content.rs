use rocket::{fairing::AdHoc, fs::NamedFile, tokio::io};

use crate::{filepath, gateway::auth::AuthState};

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Content stage", |rocket| async {
        rocket.mount("/content", routes![index])
    })
}

#[get("/")]
async fn index(auth: AuthState) -> io::Result<NamedFile> {
    let path: String;

    if auth.valid {
        path = filepath("/static/content/post.html");
    } else {
        path = filepath("/static/content/noauth.html");
    }

    NamedFile::open(path).await
}
