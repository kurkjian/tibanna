mod runner;

use runner::compile_and_run;

#[test]
fn test_while() {
    let prog = r#"
        fn main() {
            let x = 0;
            while x < 10 {
                x = x + 1;
            }
            exit(x);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(10));
}

#[test]
fn test_condition_failure() {
    let prog = r#"
        fn main() {
            let x = 0;
            while x < 0 {
                x = x + 1;
            }
            exit(x);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(0));
}

#[test]
fn test_nested() {
    let prog = r#"
        fn main() {
            let x = 0;
            let i = 0;

            while i < 3 {
                let j = 0;
                while j < 2 {
                    x = x + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            exit(x);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(6));
}
