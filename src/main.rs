use expectrl::{session, spawn, Eof};
use similar::{ChangeTag, TextDiff};
use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Output};
use std::time::Duration;
use tempfile::TempDir;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    //! Test Matrix:
    //!
    //! | Method   | Source  | Format | Function Name                                  |
    //! |----------|---------|--------|------------------------------------------------|
    //! | command  | file    | text   | test_psql_command__copy_from_file___text__     |
    //! | command  | file    | csv    | test_psql_command__copy_from_file___csv___     |
    //! | command  | file    | binary | test_psql_command__copy_from_file___binary     |
    //! | script   | stdin   | text   | test_psql_script___copy_from_stdin__text__     |
    //! | script   | stdin   | csv    | test_psql_script___copy_from_stdin__csv___     |
    //! | script   | stdin   | binary | test_psql_script___copy_from_stdin__binary     |
    //! | terminal | tty     | text   | test_psql_terminal_copy_from_tty____text__     |
    //! | terminal | tty     | csv    | test_psql_terminal_copy_from_tty____csv___     |
    //! | terminal | tty     | binary | test_psql_terminal_copy_from_tty____binary     | <-- TODO or invalid case?
    //! | terminal | stdin   | text   | test_psql_terminal_copy_from_stdin__text__     |
    //! | terminal | stdin   | csv    | test_psql_terminal_copy_from_stdin__csv___     |
    //! | terminal | stdin   | binary | test_psql_terminal_copy_from_stdin__binary     | <-- TODO or invalid case?

    use super::*;
    use once_cell::sync::OnceCell;
    use std::path::PathBuf;

    static TEST_ENVIRONMENT: OnceCell<TestEnvironment> = OnceCell::new();

    struct TestEnvironment {
        temp_dir: PathBuf,
        file_path_text: String,
        file_path_binary: String,
        file_path_csv: String,
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
            let output = run_cmd(
                "psql",
                &[
                    "-c",
                    &format!(r#"CREATE TABLE "{0}" (c1 int8, c2 int8);"#, test_table),
                ],
            )
            .unwrap();
            expect_create_table!(output);
            let output = run_cmd(
                "psql",
                &[
                    "-c",
                    &format!(
                        r#"INSERT INTO "{0}" (c1, c2) VALUES (1, 2), (3, 4);"#,
                        test_table
                    ),
                ],
            )
            .unwrap();
            expect_insert_two!(output);
            let output = run_cmd(
                "psql",
                &[
                    "-c",
                    &format!(
                        r#"\copy "{0}" to '{1}' (format text);"#,
                        test_table, file_path_text
                    ),
                ],
            )
            .unwrap();
            expect_copy_two!(output);
            let output = run_cmd(
                "psql",
                &[
                    "-c",
                    &format!(
                        r#"\copy "{0}" to '{1}' (format binary);"#,
                        test_table, file_path_binary
                    ),
                ],
            )
            .unwrap();
            expect_copy_two!(output);
            let output = run_cmd(
                "psql",
                &[
                    "-c",
                    &format!(
                        r#"\copy "{0}" to '{1}' (format csv);"#,
                        test_table, file_path_csv
                    ),
                ],
            )
            .unwrap();
            expect_copy_two!(output);
            let output =
                run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)]).unwrap();
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

    fn get_test_environment() -> &'static TestEnvironment {
        TEST_ENVIRONMENT.get_or_init(|| TestEnvironment::new())
    }

    #[test]
    fn test_psql_command__copy_from_file___binary() -> Result<(), Box<dyn Error>> {
        let env = get_test_environment();
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(
                    r#"\copy "{}" from '{}' with (format binary)"#,
                    test_table, env.file_path_binary
                ),
            ],
        )?;
        expect_copy_two!(output);
        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_command__copy_from_file___csv___() -> Result<(), Box<dyn Error>> {
        let env = get_test_environment();
        let test_table = Uuid::new_v4();
        run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(
                    r#"\copy "{}" from '{}' (format csv)"#,
                    test_table, env.file_path_csv
                ),
            ],
        )?;
        expect_copy_two!(output);
        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_command__copy_from_file___text__() -> Result<(), Box<dyn Error>> {
        let env = get_test_environment();
        let test_table = Uuid::new_v4();
        run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"\copy "{}" from '{}'"#, test_table, env.file_path_text),
            ],
        )?;
        expect_copy_two!(output);
        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_script___copy_from_stdin__binary() -> Result<(), Box<dyn Error>> {
        let env = get_test_environment();
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);
        let test_file_path = env.temp_dir.join(format!("{}", test_table));
        let mut test_file = File::create(&test_file_path)?;
        writeln!(
            test_file,
            r#"\copy "{}" from stdin with (format binary)"#,
            test_table
        )?;
        let data_content = fs::read(&env.file_path_binary)?;
        test_file.write_all(&data_content)?;
        let output = run_cmd(
            "psql",
            &["-f", &test_file_path.to_string_lossy().into_owned()],
        )?;
        expect_copy_two!(output);
        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_script___copy_from_stdin__csv___() -> Result<(), Box<dyn Error>> {
        let env = get_test_environment();
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);
        let test_file_path = env.temp_dir.join(format!("{}", test_table));
        let mut test_file = File::create(&test_file_path)?;
        writeln!(
            test_file,
            r#"\copy "{}" from stdin (format csv)"#,
            test_table
        )?;
        let data_content = fs::read_to_string(&env.file_path_csv)?;
        write!(test_file, "{}", data_content)?;
        let output = run_cmd(
            "psql",
            &["-f", &test_file_path.to_string_lossy().into_owned()],
        )?;
        expect_copy_two!(output);
        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_script___copy_from_stdin__text__() -> Result<(), Box<dyn Error>> {
        let env = get_test_environment();
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);
        let test_file_path = env.temp_dir.join(format!("{}", test_table));
        let mut test_file = File::create(&test_file_path)?;
        writeln!(test_file, r#"\copy "{}" from stdin"#, test_table)?;
        let data_content = fs::read_to_string(&env.file_path_text)?;
        write!(test_file, "{}", data_content)?;
        let output = run_cmd(
            "psql",
            &["-f", &test_file_path.to_string_lossy().into_owned()],
        )?;
        expect_copy_two!(output);
        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_terminal_copy_from_tty____text__() -> Result<(), Box<dyn Error>> {
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);

        let temp_file = tempfile::NamedTempFile::new()?;
        let log_file = temp_file.as_file();

        let mut session = session::log(spawn("psql")?, log_file.try_clone()?)?;

        session.set_expect_timeout(Some(Duration::from_secs(1)));

        let database_name =
            std::env::var("PGDATABASE").unwrap_or_else(|_| std::env::var("USER").unwrap());

        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line(&format!(r#"\copy "{}" from '/dev/tty'"#, test_table))?;
        expect!(
            &mut session,
            "Enter data to be copied followed by a newline.",
            &temp_file
        );
        expect!(
            &mut session,
            "End with a backslash and a period on a line by itself, or an EOF signal.",
            &temp_file
        );
        expect!(&mut session, ">>", &temp_file);
        session.send_line("1\t2")?;
        expect!(&mut session, ">>", &temp_file);
        session.send_line("3\t4")?;
        expect!(&mut session, ">>", &temp_file);
        // XXX Weird, `\copy ... from /dev/tty (format text)` works with or without \., contrary to (format csv) where \. gives an error
        session.send_line("\\.")?;
        expect!(&mut session, ">>", &temp_file);
        write!(session, "\x04")?;
        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line("\\q")?;
        session.expect(Eof)?;

        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_terminal_copy_from_stdin__text__() -> Result<(), Box<dyn Error>> {
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);

        let temp_file = tempfile::NamedTempFile::new()?;
        let log_file = temp_file.as_file();

        let mut session = session::log(spawn("psql")?, log_file.try_clone()?)?;

        session.set_expect_timeout(Some(Duration::from_secs(1)));

        let database_name =
            std::env::var("PGDATABASE").unwrap_or_else(|_| std::env::var("USER").unwrap());

        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line(&format!(r#"\copy "{}" from stdin"#, test_table))?;
        expect!(
            &mut session,
            "Enter data to be copied followed by a newline.",
            &temp_file
        );
        expect!(
            &mut session,
            "End with a backslash and a period on a line by itself, or an EOF signal.",
            &temp_file
        );
        expect!(&mut session, ">>", &temp_file);
        session.send_line("1\t2")?;
        expect!(&mut session, ">>", &temp_file);
        session.send_line("3\t4")?;
        expect!(&mut session, ">>", &temp_file);
        session.send_line("\\.")?;
        expect!(&mut session, "COPY 2", &temp_file);
        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line("\\q")?;
        session.expect(Eof)?;

        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_terminal_copy_from_stdin__csv___() -> Result<(), Box<dyn Error>> {
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);

        let temp_file = tempfile::NamedTempFile::new()?;
        let log_file = temp_file.as_file();

        let mut session = session::log(spawn("psql")?, log_file.try_clone()?)?;

        session.set_expect_timeout(Some(Duration::from_secs(1)));

        let database_name =
            std::env::var("PGDATABASE").unwrap_or_else(|_| std::env::var("USER").unwrap());

        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line(&format!(
            r#"\copy "{}" from stdin (format csv)"#,
            test_table
        ))?;
        expect!(
            &mut session,
            "Enter data to be copied followed by a newline.",
            &temp_file
        );
        expect!(
            &mut session,
            "End with a backslash and a period on a line by itself, or an EOF signal.",
            &temp_file
        );
        expect!(&mut session, ">>", &temp_file);
        session.send_line("1,2")?;
        expect!(&mut session, ">>", &temp_file);
        session.send_line("3,4")?;
        expect!(&mut session, ">>", &temp_file);
        session.send_line("\\.")?;
        expect!(&mut session, "COPY 2", &temp_file);
        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line("\\q")?;
        session.expect(Eof)?;

        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }

    #[test]
    fn test_psql_terminal_copy_from_tty____csv___() -> Result<(), Box<dyn Error>> {
        let test_table = Uuid::new_v4();
        let output = run_cmd(
            "psql",
            &[
                "-c",
                &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table),
            ],
        )?;
        expect_create_table!(output);

        let temp_file = tempfile::NamedTempFile::new()?;
        let log_file = temp_file.as_file();

        let mut session = session::log(spawn("psql")?, log_file.try_clone()?)?;

        session.set_expect_timeout(Some(Duration::from_secs(1)));

        let database_name =
            std::env::var("PGDATABASE").unwrap_or_else(|_| std::env::var("USER").unwrap());

        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line(&format!(
            r#"\copy "{}" from '/dev/tty' (format csv)"#,
            test_table
        ))?;
        expect!(
            &mut session,
            "Enter data to be copied followed by a newline.",
            &temp_file
        );
        expect!(
            &mut session,
            "End with a backslash and a period on a line by itself, or an EOF signal.",
            &temp_file
        );
        expect!(&mut session, ">>", &temp_file);
        session.send_line("1,2")?;
        expect!(&mut session, ">>", &temp_file);
        session.send_line("3,4")?;
        expect!(&mut session, ">>", &temp_file);
        // XXX Weird, `\copy ... from /dev/tty (format text)` works with or without \., contrary to (format csv) where \. gives an error
        // session.send_line("\\.")?;
        // expect!(&mut session, ">>", &temp_file);
        write!(session, "\x04")?;
        expect!(&mut session, &format!("{}=#", database_name), &temp_file);
        session.send_line("\\q")?;
        session.expect(Eof)?;

        let output = run_cmd(
            "psql",
            &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)],
        )?;
        expect_result_set!(output);
        let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
        expect_drop_table!(output);
        Ok(())
    }
}

fn run_cmd(program: &str, args: &[&str]) -> io::Result<Output> {
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
                write!(&mut stdout, "{}{}", sign, change).unwrap();
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
