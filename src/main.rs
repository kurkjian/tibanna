use anyhow::{Result, anyhow, bail};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Command,
};
use tibanna::{
    compile::Compiler,
    lexer::Lexer,
    parser::{Parser, Program},
};

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        bail!("Expected two arguments\nUsage: tibanna <file.tb>")
    }

    let path = Path::new(&args[1]);
    let mut file = File::open(path).map_err(|e| anyhow!(format!("Could not open file: {}", e)))?;
    let mut buf = String::new();

    file.read_to_string(&mut buf)
        .map_err(|e| anyhow!(format!("Could not read file: {}", e)))?;
    let program = parse(&mut buf)?;

    let output_path = path.parent().unwrap().join("out.s");
    let mut asm_file =
        File::create(&output_path).map_err(|e| anyhow!(format!("Could not create file: {}", e)))?;

    let instructions = Compiler::new(program).compile();
    for instr in instructions {
        asm_file
            .write(format!("{}\n", instr).as_bytes())
            .map_err(|e| anyhow!(format!("Could not write to file: {}", e)))?;
    }

    asm_file
        .flush()
        .map_err(|e| anyhow!(format!("Could not flush file: {}", e)))?;

    link()?;

    Ok(())
}

fn parse(prog: &mut str) -> Result<Program> {
    let mut lexer = Lexer::new(prog);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    parser
        .parse()
        .map_err(|e| anyhow!(format!("parse err: {e:?}")))
}

fn link() -> Result<()> {
    let _nasm = Command::new("nasm")
        .arg("-f")
        .arg("elf64")
        .arg("-o")
        .arg("out.o")
        .arg("out.s")
        .output()
        .map_err(|e| anyhow!(format!("nasm err: {e:?}")))?;

    let _ld = Command::new("ld")
        .arg("-o")
        .arg("out")
        .arg("out.o")
        .output()
        .map_err(|e| anyhow!(format!("ld err: {e:?}")))?;

    Ok(())
}
