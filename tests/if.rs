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

#[test]
fn test_lt() {
    let prog = r#"
        if 3 + 5 < 9 + 7 {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_leq() {
    let prog = r#"
        let x = 2;
        let y = 2;
        if x <= y {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_gt() {
    let prog = r#"
        let x = 5 + 3;
        let y = 9 + 7;
        if x > y {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_geq() {
    let prog = r#"
        let x = 2;
        let y = 2;
        if x >= y {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_eq() {
    let prog = r#"
        let x = 5 + 3;
        let y = 9 + 7;
        if x == y {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_neq() {
    let prog = r#"
        let x = 5 + 3;
        let y = 9 + 7;
        if x != y {
            exit(1);
        }
        exit(2);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_else() {
    let prog = r#"
        let x = 5 + 3;
        let y = 9 + 7;
        if x == y {
            exit(1);
        } else {
            exit(2);
        }
        exit(3);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_elif() {
    let prog = r#"
        let x = 5 + 3;
        let y = 9 + 7;
        if x == y {
            exit(1);
        } else if x < y {
            exit(2);
        } else {
            exit(3);
        }
        exit(4);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_elif_exclusive() {
    let prog = r#"
        let x = 1;
        let y = 1;
        let z = 0;
        if x == y {
            z = z + 1;
        } else if x < 2 {
            z = z + 1;
        } else {
            z = z + 1;
        }
        exit(z);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn test_and() {
    let prog = r#"
        let x = false;
        let y = true;
        if x && y {
            exit(1);
        } else {
            exit(2);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_or() {
    let prog = r#"
        let x = false;
        let y = true;
        if x || y {
            exit(1);
        } else {
            exit(2);
        }
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(1));
}
