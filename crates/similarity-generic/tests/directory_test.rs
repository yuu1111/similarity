use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_go_directory(dir: &TempDir) {
    let sub = dir.path().join("pkg");
    fs::create_dir_all(&sub).unwrap();

    fs::write(
        dir.path().join("main.go"),
        r#"
package main

func add(a, b int) int {
    return a + b
}

func subtract(a, b int) int {
    return a - b
}
"#,
    )
    .unwrap();

    fs::write(
        sub.join("util.go"),
        r#"
package pkg

func multiply(a, b int) int {
    return a * b
}
"#,
    )
    .unwrap();
}

#[test]
fn test_directory_walk_go() {
    let dir = TempDir::new().unwrap();
    setup_go_directory(&dir);

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.5");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates..."));
}

#[test]
fn test_directory_walk_java() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("com").join("example");
    fs::create_dir_all(&sub).unwrap();

    fs::write(
        sub.join("Calculator.java"),
        r#"
public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }
    public int sum(int x, int y) {
        return x + y;
    }
}
"#,
    )
    .unwrap();

    fs::write(
        sub.join("MathUtils.java"),
        r#"
public class MathUtils {
    public int multiply(int a, int b) {
        return a * b;
    }
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("java")
        .arg("--threshold")
        .arg("0.5");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates..."));
}

#[test]
fn test_extensions_override() {
    let dir = TempDir::new().unwrap();

    // Create a .go file and a .txt file (should be ignored by default)
    fs::write(
        dir.path().join("main.go"),
        r#"
package main
func add(a, b int) int { return a + b }
"#,
    )
    .unwrap();

    fs::write(dir.path().join("notes.txt"), "this is not Go code").unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.5");

    // Should only find the .go file
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 1 files for duplicates..."));
}

#[test]
fn test_show_functions_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("a.go"),
        r#"
package main
func alpha() {}
func beta() {}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("b.go"),
        r#"
package main
func gamma() {}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--show-functions");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"))
        .stdout(predicate::str::contains("gamma"));
}

#[test]
fn test_multiple_paths() {
    let dir = TempDir::new().unwrap();

    let file1 = dir.path().join("one.go");
    fs::write(
        &file1,
        r#"
package main
func funcA(a, b int) int { return a + b }
"#,
    )
    .unwrap();

    let file2 = dir.path().join("two.go");
    fs::write(
        &file2,
        r#"
package main
func funcB(x, y int) int { return x + y }
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(&file1)
        .arg(&file2)
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.5");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates..."));
}
