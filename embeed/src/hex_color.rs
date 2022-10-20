use std::fmt::Display;

use rocket::form::{self, FromFormField, ValueField};

pub struct HexColor<'a>(&'a str);

impl<'a> Display for HexColor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[rocket::async_trait]
impl<'a> FromFormField<'a> for HexColor<'a> {
    fn from_value(field: ValueField<'a>) -> form::Result<'a, Self> {
        if field.value.len() != 7 {
            return Err(form::Error::validation("must contain '#' and 6 hex digits"))?;
        }

        let mut chars = field.value.chars();

        if chars.next().unwrap() != '#' {
            return Err(form::Error::validation("must start with '#'"))?;
        }

        if chars.all(|c| c.is_ascii_hexdigit()) {
            Ok(HexColor(field.value))
        } else {
            Err(form::Error::validation("all 6 digits must be valid hex"))?
        }
    }
}
