use rocket::fairing::AdHoc;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Content stage", |rocket| async {
        rocket.mount("/content", routes![])
    })
}
