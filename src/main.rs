use std::{env, fs};

mod exec;

use exec::execute;

fn usage() -> std::io::Result<()> {
    eprintln!("Usage: <program> execute [in]DNA [out]RNA");
    Ok(())
}

fn main() -> std::io::Result<()> {
    if env::args().len() != 4 {
        return usage();
    }
    match &env::args().nth(1).unwrap() as &str {
        "execute" => {
            let rna = execute(&fs::read(env::args().nth(2).unwrap())?);
            fs::write(env::args().nth(3).unwrap(), rna)?;
        }
        _ => return usage(),
    }
    Ok(())
}
