extern crate failure;
extern crate ructe;

use failure::Error;
use ructe::{compile_templates, StaticFiles};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let scss_dir = base_dir.join("scss");
    let template_dir = base_dir.join("templates");
    let mut statics = StaticFiles::new(&out_dir)?;
    statics.add_files(&base_dir.join("static"))?;
    statics.add_sass_file(&scss_dir.join("home.scss"))?;
    statics.add_sass_file(&scss_dir.join("lila.scss"))?;
    compile_templates(&template_dir, &out_dir)?;
    Ok(())
}
