# Tibanna Compiler

A compiler for my own programming language. Syntactically similar to Rust, just worse, and less safe because I'm never writing a borrow checker.

## Setup

The current implementation targets x86_64 on a Linux system. This should hopefully be relaxed in the future to target more systems.

### Building
```
git clone https://github.com/kurkjian/tibanna.git
cd tibanna
cargo build
```

Compiler executable is located at `target/debug/tibanna`

### Usage
Requires `nasm` and `ld` to create a runnable executable from the Tibanna source code:
```
./target/debug/tibanna <tibanna_source.tb>
```

This will generate an x86_64 asm file and link it to create a runnable executable (`out`).

## Example Tibanna Code
```rust
// modified from `test_nested` in `tests/while.rs`

fn inc(a: int): int {
    return a + 1;
}

fn main() {
    let x = 0;
    let i = 0;

    while i < 3 {
        let j = 0;
        while j < 2 {
            x = inc(x);
            j = inc(j);
        }
        i = inc(i);
    }

    exit(x); // i dont have a `print` implemented right now
}
```

Additional examples can be found in the `tests/` directory.

## Architecture
\<todo>

source -> lexer -> parser -> semantic analysis -> ir generation -> optimization -> codegen
