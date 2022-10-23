use std::{borrow::Cow, fmt::Display};

use rocket::{
    data::ToByteUnit,
    form::{self, DataField, FromFormField, ValueField},
};

pub struct HexColor<'a>(pub Cow<'a, str>);

impl<'a> Display for HexColor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[rocket::async_trait]
impl<'a> FromFormField<'a> for HexColor<'a> {
    fn from_value(field: ValueField<'a>) -> form::Result<'a, Self> {
        if field.value.len() == 0 {
            return Ok(Self(Cow::Borrowed("")));
        } else if field.value.len() != 7 {
            return Err(form::Error::validation("must contain exactly 7 characters"))?;
        }

        let mut chars = field.value.chars();

        if chars.next().unwrap() != '#' {
            return Err(form::Error::validation("must start with '#'"))?;
        }

        if chars.all(|c| c.is_ascii_hexdigit()) {
            Ok(HexColor(Cow::Owned(String::new())))
        } else {
            Err(form::Error::validation("all 6 digits must be valid hex"))?
        }
    }

    async fn from_data(field: DataField<'a, '_>) -> form::Result<'a, Self> {
        let limit = field
            .request
            .limits()
            .get("color")
            .unwrap_or(256.kibibytes());
        let bytes = field.data.open(limit).into_bytes().await?;

        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        let bytes = bytes.into_inner();

        if bytes.len() == 0 {
            return Ok(Self(Cow::Owned(String::new())));
        } else if bytes.len() != 7 {
            return Err(form::Error::validation("must contain exactly 7 characters"))?;
        }

        let mut bytes_iter = bytes.iter();

        if bytes_iter.next().unwrap() != &b'#' {
            Err(form::Error::validation("must start with '#'"))?
        } else if bytes_iter.all(|b| b.is_ascii_hexdigit()) {
            let stringified = String::from_utf8(bytes).unwrap();

            Ok(Self(Cow::Owned(stringified)))
        } else {
            Err(form::Error::validation("all 6 digits must be valid hex"))?
        }
    }

    fn default() -> Option<HexColor<'a>> {
        Some(Self(Cow::Owned(String::new())))
    }
}
