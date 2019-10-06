use crate::templates::statics::STATICS;
use brotli::enc::backward_references::BrotliEncoderParams;
use brotli::BrotliCompress;
use flate2::{Compression, GzBuilder};
use http::StatusCode;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let out_dir: &Path = "public".as_ref();
    create_dir_all(out_dir)?;
    File::create(out_dir.join("robots.txt"))?;
    {
        let mut f = File::create(out_dir.join("index.html"))?;
        templates::index(&mut f)?;
    }
    {
        let gifta = out_dir.join("gifta");
        create_dir_all(&gifta)?;
        let mut f = File::create(gifta.join("index.html"))?;
        templates::gifta(&mut f)?;
    }
    let dir = out_dir.join("s");
    for s in STATICS {
        println!("Handle {:?}", s.name);
        // s.name may contain directory components.
        if let Some(parent) = dir.join(s.name).parent() {
            create_dir_all(parent)?;
        }
        File::create(dir.join(s.name))
            .and_then(|mut f| f.write(s.content))?;

        let limit = s.content.len() - 10; // Compensate a few bytes overhead
        let gz = gzipped(s.content)?;
        if gz.len() < limit {
            File::create(dir.join(format!("{}.gz", s.name)))
                .and_then(|mut f| f.write(&gz))?;
        }
        let br = brcompressed(s.content)?;
        if br.len() < limit {
            File::create(dir.join(format!("{}.br", s.name)))
                .and_then(|mut f| f.write(&br))?;
        }
    }
    {
        let code = StatusCode::NOT_FOUND;
        let mut f =
            File::create(out_dir.join(format!("{}.html", code.as_u16())))?;
        templates::error(
            &mut f,
            code,
            "The resource you requested could not be located.",
        )?;
    }
    Ok(())
}

fn gzipped(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = Vec::new();
    {
        let mut gz = GzBuilder::new().write(&mut buf, Compression::best());
        gz.write_all(data)?;
        gz.finish()?;
    }
    Ok(buf)
}

fn brcompressed(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut params = BrotliEncoderParams::default();
    params.quality = 11;
    BrotliCompress(&mut data.as_ref(), &mut buf, &params)?;
    Ok(buf)
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
