use rocket::fs::NamedFile;
use rocket::tokio::io::{self, AsyncReadExt};

pub async fn read_password<'a>() -> io::Result<(String, String)> {
    let mut cred_file = NamedFile::open("creds").await?;
    let mut buf: [u8; 107] = [0; 107];

    cred_file.read(&mut buf).await.unwrap();

    let first = Vec::from(&buf[..16]);
    let second = Vec::from(&buf[17..]);

    let salt = String::from_utf8(first).unwrap();
    let hash = String::from_utf8(second).unwrap();

    Ok((salt, hash))
}
