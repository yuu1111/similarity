use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_print_flag() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("test.go"),
        r#"
package main

func add(a, b int) int {
    return a + b
}

func sum(x, y int) int {
    return x + y
}
"#,
    )
    .unwrap();

    // Without --print: should NOT show code
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("return a + b").not());

    // With --print: should show code
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7")
        .arg("--print");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("return a + b").or(predicate::str::contains("return x + y")));
}

#[test]
fn test_min_lines_filter() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("test.go"),
        r#"
package main

func tiny(a int) int { return a }
func small(b int) int { return b }

func larger(a, b int) int {
    result := a + b
    if result < 0 {
        result = 0
    }
    return result
}

func bigger(x, y int) int {
    result := x + y
    if result < 0 {
        result = 0
    }
    return result
}
"#,
    )
    .unwrap();

    // With --min-lines 5: tiny/small should be filtered out
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7")
        .arg("--min-lines")
        .arg("5");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tiny").not())
        .stdout(predicate::str::contains("small").not());
}

#[test]
fn test_fail_on_duplicates_exit_code() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("test.go"),
        r#"
package main

func add(a, b int) int {
    return a + b
}

func sum(x, y int) int {
    return x + y
}
"#,
    )
    .unwrap();

    // Without --fail-on-duplicates: should succeed
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7");
    cmd.assert().success();

    // With --fail-on-duplicates: should fail with exit code 1
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7")
        .arg("--fail-on-duplicates");
    cmd.assert().failure().code(1);
}

#[test]
fn test_fail_on_duplicates_no_duplicates() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("test.go"),
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

    // With --fail-on-duplicates but no duplicates: should succeed
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.95")
        .arg("--fail-on-duplicates");
    cmd.assert().success();
}

#[test]
fn test_exclude_pattern() {
    let dir = TempDir::new().unwrap();
    let vendor = dir.path().join("vendor");
    fs::create_dir_all(&vendor).unwrap();

    fs::write(
        dir.path().join("main.go"),
        r#"
package main

func add(a, b int) int {
    return a + b
}
"#,
    )
    .unwrap();

    fs::write(
        vendor.join("lib.go"),
        r#"
package vendor

func vendorFunc(a, b int) int {
    return a + b
}
"#,
    )
    .unwrap();

    // Without exclude: should find 2 files
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.5");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 2 files"));

    // With exclude vendor: should find 1 file
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.5")
        .arg("--exclude")
        .arg("vendor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Checking 1 files"));
}

#[test]
fn test_filter_function_name() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("test.go"),
        r#"
package main

func addNumbers(a, b int) int {
    return a + b
}

func sumNumbers(x, y int) int {
    return x + y
}

func multiply(a, b int) int {
    return a * b
}

func product(x, y int) int {
    return x * y
}
"#,
    )
    .unwrap();

    // Filter for "add": should only show pairs containing "add"
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.7")
        .arg("--filter-function")
        .arg("Numbers");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Numbers"));
}

#[test]
fn test_no_files_found() {
    let dir = TempDir::new().unwrap();

    // Empty directory
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.8");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No files found"));
}

#[test]
fn test_rename_cost_option() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("test.go"),
        r#"
package main

func add(a, b int) int {
    return a + b
}

func sum(x, y int) int {
    return x + y
}
"#,
    )
    .unwrap();

    // With very high rename cost, similarity should drop
    let mut cmd = Command::cargo_bin("similarity-generic").unwrap();
    cmd.arg(dir.path())
        .arg("--language")
        .arg("go")
        .arg("--threshold")
        .arg("0.99")
        .arg("--rename-cost")
        .arg("1.0");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No duplicate functions found!"));
}
