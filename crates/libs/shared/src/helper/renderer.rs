use anyhow::Error;
use tera::{Context, Tera};

pub fn render_string(tera_template: &str, locals: &serde_json::Value) -> Result<String, Error> {
    let text = Tera::one_off(tera_template, &Context::from_serialize(locals)?, false)?;
    Ok(text)
}
