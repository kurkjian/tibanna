mod runner;

use runner::compile_and_run;

#[test]
fn test_if_false() {
    let prog = r#"
        if 3 + 2 {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_if_true() {
    let prog = r#"
        if 5 - 5 {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_nested() {
    let prog = r#"
        if 3 + 5 {
            if 9 + 7 {
                let x = 1;
                let y = 2;
            }
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}
