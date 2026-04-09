use anyhow::{Result, anyhow, bail};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
};

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        bail!("Expected two arguments\nUsage: tibanna <file.tb>")
    }

    let path = Path::new(&args[1]);
    let exit_code = parse(path)?;

    let output_path = path.parent().unwrap().join("out.s");
    let mut asm_file =
        File::create(&output_path).map_err(|e| anyhow!(format!("Could not create file: {}", e)))?;
    asm_file
        .write(code_gen(exit_code).as_bytes())
        .map_err(|e| anyhow!(format!("Could not write to file: {}", e)))?;
    asm_file
        .flush()
        .map_err(|e| anyhow!(format!("Could not flush file: {}", e)))?;

    Ok(())
}

fn parse(p: &Path) -> Result<u8> {
    let mut file = File::open(p).map_err(|e| anyhow!(format!("Could not open file: {}", e)))?;
    let mut buf = String::new();

    file.read_to_string(&mut buf)
        .map_err(|e| anyhow!(format!("Could not read file: {}", e)))?;

    let mut split = buf.split(" ");
    if let Some(token) = split.next() {
        let next = split.next().map(|x| x.trim().parse::<u8>());
        if token == "exit" && next.as_ref().is_some_and(|f| f.is_ok()) {
            return Ok(next.unwrap().unwrap());
        }

        bail!("Failed to parse valid program")
    } else {
        bail!("Failed to find any tokens in file")
    }
}

fn code_gen(exit_code: u8) -> String {
    format!(
        "\
        global _start
_start:
   move rax, 60
   move rdi, {exit_code}
   syscall
        "
    )
}
