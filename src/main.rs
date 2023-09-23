use std::{io::Read, fs::File, env};

use anyhow::Result;

use cpu::Cpu;

mod cpu;
mod mem;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: rue <filename>");
    }

    let mut file = File::open(&args[1])?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    
    Cpu::from_buf(buf).run()?;
    
    Ok(())
}

