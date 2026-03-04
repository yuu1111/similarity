use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cross_file_go() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("math.go"),
        r#"
package main

func addNumbers(a, b int) int {
    result := a + b
    return result
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("utils.go"),
        r#"
package main

func sumValues(x, y int) int {
    result := x + y
    return result
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates..."))
        .stdout(predicate::str::contains("addNumbers").or(predicate::str::contains("sumValues")));
}

#[test]
fn test_cross_file_java() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("Adder.java"),
        r#"
public class Adder {
    public int calculate(int a, int b) {
        int result = a + b;
        return result;
    }
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("Summer.java"),
        r#"
public class Summer {
    public int compute(int x, int y) {
        int result = x + y;
        return result;
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
        .arg("0.7");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files for duplicates..."))
        .stdout(
            predicate::str::contains("calculate").or(predicate::str::contains("compute")),
        );
}

#[test]
fn test_cross_file_shows_both_file_paths() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("a.go"),
        r#"
package main

func doWork(a, b int) int {
    result := a + b
    return result
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("b.go"),
        r#"
package main

func doTask(x, y int) int {
    result := x + y
    return result
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7");

    // Both file names should appear in output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("a.go"))
        .stdout(predicate::str::contains("b.go"));
}

#[test]
fn test_no_cross_file_when_different() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("math.go"),
        r#"
package main

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("string.go"),
        r#"
package main

import "strings"

func reverseString(s string) string {
    runes := []rune(s)
    for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
        runes[i], runes[j] = runes[j], runes[i]
    }
    return strings.Join(nil, "")
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.9");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}
