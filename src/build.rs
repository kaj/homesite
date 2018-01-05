extern crate ructe;

use ructe::{compile_templates, StaticFiles};
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let scss_dir = base_dir.join("scss");
    let template_dir = base_dir.join("templates");
    let mut statics = StaticFiles::new(&out_dir).unwrap();
    statics.add_files(&base_dir.join("static")).unwrap();
    statics.add_sass_file(&scss_dir.join("home.scss")).unwrap();
    statics.add_sass_file(&scss_dir.join("lila.scss")).unwrap();
    compile_templates(&template_dir, &out_dir).expect("templates");
}
