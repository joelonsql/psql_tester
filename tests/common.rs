use similar::{ChangeTag, TextDiff};
use std::borrow::Cow;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;
use once_cell::sync::OnceCell;

#[macro_export]
macro_rules! verify {
    ($content:expr, $expected_str:expr) => {{
        let content = $content;
        let content_str = String::from_utf8_lossy(&content);
        let expected = if $expected_str.starts_with('\n') {
            Cow::Borrowed(&$expected_str[1..])
        } else {
            Cow::Borrowed($expected_str)
        };

        if content_str != expected {
            println!("\nUnexpected output at {}:{}", file!(), line!());
            let diff = TextDiff::from_lines(&content_str, &expected);

            let mut stdout = StandardStream::stdout(ColorChoice::Always);

            for change in diff.iter_all_changes() {
                let (sign, color) = match change.tag() {
                    ChangeTag::Delete => (
                        "-",
                        ColorSpec::new().set_fg(Some(termcolor::Color::Red)).clone(),
                    ),
                    ChangeTag::Insert => (
                        "+",
                        ColorSpec::new()
                            .set_fg(Some(termcolor::Color::Green))
                            .clone(),
                    ),
                    ChangeTag::Equal => (" ", ColorSpec::new().clone()),
                };

                stdout.set_color(&color).unwrap();
                let _ = stdout.write_all(sign.as_bytes());
                let _ = stdout.write_all(change.to_string().as_bytes());
                stdout.reset().unwrap();
            }
            panic!("Verification failed");
        }
    }};
}

#[macro_export]
macro_rules! expect {
    ($session:expr, $pattern:expr, $log_file:expr) => {{
        if let Err(_) = $session.expect($pattern) {
            let logs = std::fs::read_to_string($log_file.path()).unwrap();
            println!("Unexpected output at {}:{}", file!(), line!());
            println!("Session logs at time of failure:\n{}", logs);
            println!("Failed to find expected pattern: {}", $pattern);
            panic!("Expectation failed");
        }
    }};
}

#[macro_export]
macro_rules! isempty {
    ($content:expr) => {{
        verify!($content, "\n");
    }};
}

#[macro_export]
macro_rules! expect_copy_two {
    ($output:expr) => {{
        verify!(
            $output.stdout,
            r#"
COPY 2
"#
        );
        isempty!($output.stderr);
    }};
}

#[macro_export]
macro_rules! expect_insert_two {
    ($output:expr) => {{
        verify!(
            $output.stdout,
            r#"
INSERT 0 2
"#
        );
        isempty!($output.stderr);
    }};
}

#[macro_export]
macro_rules! expect_create_table {
    ($output:expr) => {{
        verify!(
            $output.stdout,
            r#"
CREATE TABLE
"#
        );
        isempty!($output.stderr);
    }};
}

#[macro_export]
macro_rules! expect_drop_table {
    ($output:expr) => {{
        verify!(
            $output.stdout,
            r#"
DROP TABLE
"#
        );
        isempty!($output.stderr);
    }};
}

#[macro_export]
macro_rules! expect_result_set {
    ($output:expr) => {{
        verify!($output.stdout, r#"
 c1 | c2 
----+----
  1 |  2
  3 |  4
(2 rows)

"#);
        isempty!($output.stderr);
    }};
}

static TEST_ENVIRONMENT: OnceCell<TestEnvironment> = OnceCell::new();

pub struct TestEnvironment {
    pub temp_dir: PathBuf,
    pub file_path_text: String,
    pub file_path_binary: String,
    pub file_path_csv: String,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap().into_path();
        let test_table = Uuid::new_v4();
        let base_file = temp_dir.join(test_table.to_string());
        let file_path_text = base_file
            .with_extension("text")
            .to_string_lossy()
            .into_owned();
        let file_path_binary = base_file
            .with_extension("binary")
            .to_string_lossy()
            .into_owned();
        let file_path_csv = base_file
            .with_extension("csv")
            .to_string_lossy()
            .into_owned();

        let output = run_cmd("psql", &["-c", &format!(r#"CREATE TABLE "{0}" (c1 int8, c2 int8);"#, test_table)]).unwrap();
        expect_create_table!(output);

        let output = run_cmd("psql", &["-c", &format!(r#"INSERT INTO "{0}" (c1, c2) VALUES (1, 2), (3, 4);"#, test_table)]).unwrap();
        expect_insert_two!(output);

        let output = run_cmd("psql", &["-c", &format!(r#"\copy "{0}" to '{1}' (format text);"#, test_table, file_path_text)]).unwrap();
        expect_copy_two!(output);

        let output = run_cmd("psql", &["-c", &format!(r#"\copy "{0}" to '{1}' (format binary);"#, test_table, file_path_binary)]).unwrap();
        expect_copy_two!(output);

        let output = run_cmd("psql", &["-c", &format!(r#"\copy "{0}" to '{1}' (format csv);"#, test_table, file_path_csv)]).unwrap();
        expect_copy_two!(output);

        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)]).unwrap();
        expect_drop_table!(output);

        Self {
            temp_dir,
            file_path_text,
            file_path_binary,
            file_path_csv,
        }
    }

    fn cleanup(&self) {
        fs::remove_dir_all(&self.temp_dir).unwrap();
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        self.cleanup();
    }
}

pub fn get_test_environment() -> &'static TestEnvironment {
    TEST_ENVIRONMENT.get_or_init(|| TestEnvironment::new())
}

pub fn run_cmd(program: &str, args: &[&str]) -> io::Result<Output> {
    let output = Command::new(program).args(args).output()?;

    if !output.status.success() {
        println!("Failed command: {} {}", program, args.join(" "));
        if !output.stdout.is_empty() {
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(output)
}