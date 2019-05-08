use ructe::{Ructe, RucteError};

fn main() -> Result<(), RucteError> {
    let mut ructe = Ructe::from_env()?;
    let mut statics = ructe.statics()?;
    statics.add_sass_file("scss/home.scss")?;
    statics.add_sass_file("scss/lila.scss")?;
    statics.add_files("static")?;
    ructe.compile_templates("templates")
}
