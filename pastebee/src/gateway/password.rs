use rocket::data::ToByteUnit;
use rocket::form::{self, DataField, FromFormField, ValueField};
use rocket::fs::NamedFile;
use rocket::tokio::io::{self, AsyncReadExt};

#[derive(Hash)]
pub struct Password(pub String);

pub async fn read_password() -> io::Result<String> {
    let mut password = String::new();
    let mut cred_file = NamedFile::open("creds").await?;

    cred_file.read_to_string(&mut password).await?;

    Ok(password)
}

#[rocket::async_trait]
impl<'a> FromFormField<'a> for Password {
    fn from_value(field: ValueField<'a>) -> form::Result<'a, Self> {
        Ok(Password(field.value.to_owned()))
    }

    async fn from_data(field: DataField<'a, '_>) -> form::Result<'a, Self> {
        let limit = field
            .request
            .limits()
            .get("password")
            .unwrap_or(256.kibibytes());
        let bytes = field.data.open(limit).into_bytes().await?;

        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        let bytes = bytes.into_inner();
        let password = String::from_utf8(bytes);

        if password.is_err() {
            Err(form::Error::validation("that is not valid utf-8"))?
        } else {
            let password = password.unwrap();

            Ok(Password(password))
        }
    }
}
