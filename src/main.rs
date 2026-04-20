use anyhow::{Result, anyhow, bail};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
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
    let program = parse(path)?;

    let output_path = path.parent().unwrap().join("out.s");
    let mut asm_file =
        File::create(&output_path).map_err(|e| anyhow!(format!("Could not create file: {}", e)))?;
    init_header(&mut asm_file)?;

    let instructions = Compiler::new().compile(program);
    for instr in instructions {
        asm_file
            .write(format!("{}\n", instr).as_bytes())
            .map_err(|e| anyhow!(format!("Could not write to file: {}", e)))?;
    }

    asm_file
        .flush()
        .map_err(|e| anyhow!(format!("Could not flush file: {}", e)))?;

    Ok(())
}

fn parse(p: &Path) -> Result<Program> {
    let mut file = File::open(p).map_err(|e| anyhow!(format!("Could not open file: {}", e)))?;
    let mut buf = String::new();

    file.read_to_string(&mut buf)
        .map_err(|e| anyhow!(format!("Could not read file: {}", e)))?;

    let mut lexer = Lexer::new(&buf);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    Ok(program)
}

fn init_header(asm_file: &mut File) -> Result<()> {
    asm_file
        .write(
            "\
            global _start
_start:
"
            .to_string()
            .as_bytes(),
        )
        .map_err(|e| anyhow!(format!("Could not write to file: {}", e)))?;
    Ok(())
}
