use rand::{self, Rng};
use rocket::data::ToByteUnit;
use rocket::form::{self, DataField, FromFormField, ValueField};
use rocket::request::FromParam;
use std::borrow::Cow;

#[derive(Clone, UriDisplayPath)]
pub struct EmbedId<'a>(pub Cow<'a, str>);

impl<'a> EmbedId<'a> {
    pub fn new(size: usize) -> EmbedId<'static> {
        const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        let mut id = String::with_capacity(size);
        let mut rng = rand::thread_rng();

        for _ in 0..size {
            id.push(BASE62[rng.gen::<usize>() % 62] as char);
        }

        EmbedId(Cow::Owned(id))
    }
}

impl<'a> FromParam<'a> for EmbedId<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            .then(|| EmbedId(param.into()))
            .ok_or(param)
    }
}

#[rocket::async_trait]
impl<'a> FromFormField<'a> for EmbedId<'a> {
    fn from_value(field: ValueField<'a>) -> form::Result<'a, Self> {
        let valid = field
            .value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

        if valid {
            Ok(EmbedId(Cow::Borrowed(field.value)))
        } else {
            Err(form::Error::validation(
                "can only contain A-Z, a-z, 0-9, -, _",
            ))?
        }
    }

    async fn from_data(field: DataField<'a, '_>) -> form::Result<'a, Self> {
        let limit = field.request.limits().get("id").unwrap_or(256.kibibytes());
        let bytes = field.data.open(limit).into_bytes().await?;

        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        let bytes = bytes.into_inner();

        if bytes
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || c == &b'-' || c == &b'_')
        {
            let stringified = String::from_utf8(bytes).unwrap();

            if stringified.len() == 0 {
                Err(form::Error::validation("id cannot be empty"))?
            }

            Ok(EmbedId(Cow::Owned(stringified)))
        } else {
            Err(form::Error::validation(
                "can only contain A-Z, a-z, 0-9, -, _",
            ))?
        }
    }
}
