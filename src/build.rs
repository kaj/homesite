use ructe::{Ructe, RucteError};

fn main() -> Result<(), RucteError> {
    let mut ructe = Ructe::from_env()?;
    let mut statics = ructe.statics()?;
    statics.add_files("static")?;
    statics.add_sass_file("scss/home.scss")?;
    statics.add_sass_file("scss/lila.scss")?;
    ructe.compile_templates("templates")
}
