use std::borrow::Cow;
use std::path::{Path, PathBuf};

use rand::{self, Rng};
use rocket::form::{self, DataField, FromFormField, ValueField};
use rocket::request::FromParam;

#[derive(UriDisplayPath)]
pub struct EmbedId<'a>(Cow<'a, str>);

impl EmbedId<'_> {
    pub fn new(size: usize) -> EmbedId<'static> {
        const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        let mut id = String::with_capacity(size);
        let mut rng = rand::thread_rng();

        for _ in 0..size {
            id.push(BASE62[rng.gen::<usize>() % 62] as char);
        }

        EmbedId(Cow::Owned(id))
    }

    pub fn file_path(&self) -> PathBuf {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/", "upload");
        Path::new(root).join(self.0.as_ref())
    }
}

impl<'a> FromParam<'a> for EmbedId<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
            .then(|| EmbedId(param.into()))
            .ok_or(param)
    }
}

#[rocket::async_trait]
impl<'a> FromFormField<'a> for EmbedId<'a> {
    fn from_value(field: ValueField<'a>) -> form::Result<'a, Self> {
        let valid = field.value.chars().all(|c| c.is_ascii_alphanumeric());

        if valid {
            Ok(EmbedId(Cow::Borrowed(field.value)))
        } else {
            Err(form::Error::validation("fuck you"))
        }
    }

    async fn from_data(field: DataField<'a, '_>) -> form::Result<'a, Self> {
        todo!("parse embed id");
    }
}
