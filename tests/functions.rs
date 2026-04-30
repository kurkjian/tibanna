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
