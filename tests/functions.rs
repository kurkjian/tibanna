mod runner;

use runner::compile_and_run;

#[test]
fn test_it_works() {
    let prog = r#"
        fn main() {
            let x = 1;
            exit(x);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_function_call() {
    let prog = r#"
        fn main() {
            inc_and_exit(0);
        }

        fn inc_and_exit(x: int) {
            exit(x + 1);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_function_return() {
    let prog = r#"
        fn main() {
            let y = inc_and_ret(0);
            exit(y);
        }

        fn inc_and_ret(x: int) = int {
            return x + 1;
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_main_entry() {
    let prog = r#"
        fn inc_and_ret(x: int) = int {
            return x + 1;
        }

        fn main() {
            let y = inc_and_ret(0);
            exit(y);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
#[should_panic(expected = "not yet implemented: Support lib files that don't have a main function")]
fn test_no_main_panics() {
    let prog = r#"
        fn inc_and_ret() = int {
            exit(1);
        }

    "#;

    let _ = compile_and_run(prog);
}
