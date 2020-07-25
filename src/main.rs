use std::{env, fs};

mod build;
mod cli_main;
mod exec;
mod png_utils;
mod types;

use build::build;
use cli_main::cli_main;
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
    let arg0 = env::args().nth(2).unwrap();
    let arg1 = env::args().nth(3).unwrap();
    match &command[..] {
        "execute" => {
            let rna = execute(b"IIPIFFCPICICIICPIICIPPPICIIC", &fs::read(arg0)?);
            fs::write(arg1, to_u8_vec(&rna))?;
        }
        "build" => {
            let bitmap = build(&fs::read(arg0)?);
            write_bitmap_as_png(&bitmap, fs::File::create(arg1)?)?;
        }
        "cli" => {
            cli_main(arg0.as_bytes(), &fs::read(arg1)?)?;
        }
        _ => return usage(),
    }
    Ok(())
}
