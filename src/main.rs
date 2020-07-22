use std::{env, fs};

mod build;
mod exec;
mod png_utils;

use build::build;
use exec::execute;
use png_utils::write_bitmap_as_png;

fn usage() -> std::io::Result<()> {
    eprintln!("Usage:
  <program> execute [in]DNA [out]RNA
  <program> build [in]RNA [out]PNG");
    Ok(())
}

fn main() -> std::io::Result<()> {
    if env::args().len() != 4 {
        return usage();
    }
    let command = env::args().nth(1).unwrap();
    let infile = env::args().nth(2).unwrap();
    let outfile = env::args().nth(3).unwrap();
    match &command[..] {
        "execute" => {
            let rna = execute(b"", &fs::read(infile)?);
            fs::write(outfile, rna)?;
        }
        "build" => {
            let bitmap = build(&fs::read(infile)?);
            write_bitmap_as_png(&bitmap, fs::File::create(outfile)?)?;
        }
        _ => return usage(),
    }
    Ok(())
}
