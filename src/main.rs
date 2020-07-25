use std::{env, fs};

mod build;
mod exec;
mod png_utils;
mod types;

use build::build;
use exec::execute;
use png_utils::write_bitmap_as_png;
use types::to_u8_vec;

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
    let file0 = env::args().nth(2).unwrap();
    let file1 = env::args().nth(3).unwrap();
    match &command[..] {
        "execute" => {
            let rna = execute(b"IIPIFFCPICICIICPIICIPPPICIIC", &fs::read(file0)?);
            fs::write(file1, to_u8_vec(&rna))?;
        }
        "build" => {
            let bitmap = build(&fs::read(file0)?);
            write_bitmap_as_png(&bitmap, fs::File::create(file1)?)?;
        }
        _ => return usage(),
    }
    Ok(())
}
