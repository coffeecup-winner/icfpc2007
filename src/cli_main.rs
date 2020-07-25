use std::io::{self, BufRead, Write};

use crate::build::BuilderState;
use crate::exec::ExecutionState;
use crate::png_utils::*;

enum Mode {
    DNA,
    RNA,
}

pub fn cli_main(prefix: &[u8], dna: &[u8]) -> io::Result<()> {
    let mut exec_state = ExecutionState::new(prefix, dna);
    let mut build_state: BuilderState = BuilderState::new(&[]);
    println!(
        "DNA loaded: {} bases, {} base prefix",
        dna.len(),
        prefix.len()
    );
    let mut dna_processed = false;
    let mut last_command = String::new();
    let mut mode = Mode::DNA;
    let mut last_rna_count = 0;
    loop {
        match mode {
            Mode::DNA => print!("dna> "),
            Mode::RNA => print!("rna> "),
        }
        io::stdout().lock().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if line.trim() == "" {
            line = last_command.clone();
        }
        let parts = line.trim().split_ascii_whitespace().collect::<Vec<_>>();
        match parts[0] {
            "quit" | "q" => break,
            "step" | "s" => {
                let mut num_steps = 1;
                if parts.len() > 1 {
                    num_steps = parts[1].parse::<u32>().unwrap();
                }
                match mode {
                    Mode::DNA => {
                        if !dna_processed {
                            exec_state.enable_debug_prints = false;
                            for _ in 0..num_steps - 1 {
                                exec_state.step();
                            }
                            exec_state.enable_debug_prints = true;
                            dna_processed = !exec_state.step();
                        }
                        if exec_state.rna.len() > last_rna_count {
                            build_state.extend(&exec_state.rna[last_rna_count..]);
                            println!(
                                "New RNA generated: {} commands",
                                (exec_state.rna.len() - last_rna_count) / 7
                            );
                            last_rna_count = exec_state.rna.len();
                            mode = Mode::RNA;
                        }
                    }
                    Mode::RNA => {
                        if (build_state.iteration as usize) < build_state.commands.len() {
                            num_steps = num_steps
                                .min(build_state.commands.len() as u32 - build_state.iteration);
                            build_state.enable_debug_prints = false;
                            for _ in 0..num_steps - 1 {
                                build_state.step();
                            }
                            build_state.enable_debug_prints = true;
                            let mut bitmap = build_state.step().clone();
                            build_state.draw_debug_overlay(&mut bitmap);
                            write_bitmap_as_png_rgba(
                                &bitmap,
                                std::fs::File::create("./current.png")?,
                            )?;
                            if build_state.iteration as usize == build_state.commands.len() {
                                println!("RNA exhausted, switching to DNA mode");
                                mode = Mode::DNA;
                            }
                        }
                    }
                }
            }
            "until" | "u" => match mode {
                Mode::DNA => {
                    if !dna_processed {
                        exec_state.enable_debug_prints = true;
                        while exec_state.rna.len() == last_rna_count && !dna_processed {
                            dna_processed = !exec_state.step();
                        }
                    }
                    if exec_state.rna.len() > last_rna_count {
                        build_state.extend(&exec_state.rna[last_rna_count..]);
                        println!(
                            "New RNA generated: {} commands",
                            (exec_state.rna.len() - last_rna_count) / 7
                        );
                        last_rna_count = exec_state.rna.len();
                        mode = Mode::RNA;
                    }
                }
                Mode::RNA => {
                    let num_steps = build_state.commands.len() as u32 - build_state.iteration;
                    build_state.enable_debug_prints = false;
                    for _ in 0..num_steps - 1 {
                        build_state.step();
                    }
                    build_state.enable_debug_prints = true;
                    let mut bitmap = build_state.step().clone();
                    build_state.draw_debug_overlay(&mut bitmap);
                    write_bitmap_as_png_rgba(
                        &bitmap,
                        std::fs::File::create("./current.png")?,
                    )?;
                    println!("RNA exhausted, switching to DNA mode");
                    mode = Mode::DNA;
                }
            },
            "dump" | "d" => {
                for (i, b) in build_state.bitmaps.iter().enumerate() {
                    write_bitmap_as_png_rgba(
                        b,
                        std::fs::File::create(format!("bitmap_{}.png", i))?,
                    )?;
                }
            }
            _ => {
                println!("Unknown command: {}", line);
                continue;
            }
        }
        last_command = line.trim().to_string();
    }
    Ok(())
}
