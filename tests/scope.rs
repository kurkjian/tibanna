mod runner;

use runner::compile_and_run;

#[test]
fn test_variable_shadow() {
    let prog = r#"
        fn main() {
            let x = 1;
            if x > 0 {
                let x = 2;
            }

            exit(x);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_variable_shadow_different_type() {
    let prog = r#"
        fn main() {
            let x = 1;
            if x > 0 {
                let x = false;
            }

            exit(x);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_function_scope() {
    let prog = r#"
        fn main() {
            let x = 5;
            foo();
            exit(x);
        }

        fn foo() {
            let x = false;
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(5));
}
