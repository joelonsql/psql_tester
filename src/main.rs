use expectrl::{spawn, Eof, session};
use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Output};
use std::fs;
use std::error::Error;
use std::net::TcpListener;
use tempfile::TempDir;
use std::path::Path;
use std::time::Duration;
use similar::{ChangeTag, TextDiff};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::borrow::Cow;

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
                    ChangeTag::Delete => ("-", ColorSpec::new().set_fg(Some(termcolor::Color::Red)).clone()),
                    ChangeTag::Insert => ("+", ColorSpec::new().set_fg(Some(termcolor::Color::Green)).clone()),
                    ChangeTag::Equal => (" ", ColorSpec::new().clone()),
                };
                
                stdout.set_color(&color).unwrap();
                write!(&mut stdout, "{}{}", sign, change).unwrap();
                stdout.reset().unwrap();
            }
            return Ok(false);
        }
    }};
}

macro_rules! expect {
    ($session:expr, $pattern:expr, $log_file:expr) => {{
        if let Err(_) = $session.expect($pattern) {
            // Print logs and location information
            let logs = std::fs::read_to_string($log_file.path())?;
            println!("Unexpected output at {}:{}", file!(), line!());
            println!("Session logs at time of failure:\n{}", logs);
            println!("Failed to find expected pattern: {}", $pattern);
            return Ok(false);
        }
    }};
}

macro_rules! isempty {
    ($content:expr) => {{
        verify!($content, "\n");
    }};
}

macro_rules! verify_copy_two {
    ($output:expr) => {{
        verify!($output.stdout, r#"
COPY 2
"#);
        isempty!($output.stderr);
    }};
}

macro_rules! verify_result_set {
    ($output:expr) => {{
        verify!($output.stdout, r#"
 q1 | q2 
----+----
  1 |  2
  3 |  4
(2 rows)

"#);
        isempty!($output.stderr);
    }};
}

fn main() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?.into_path();
    let port = find_available_port()?;
    setup_postgres(&temp_dir, &port)?;
    prepare_test_database(&port)?;
    generate_test_files(&temp_dir, &port)?;
    
    let mut all_tests_passed = true;
    all_tests_passed &= test_case_1(&temp_dir, &port)?;
    all_tests_passed &= test_case_2(&temp_dir, &port)?;
    all_tests_passed &= test_case_3(&temp_dir, &port)?;
    all_tests_passed &= test_case_4(&temp_dir, &port)?;
    all_tests_passed &= test_case_5(&temp_dir, &port)?;
    all_tests_passed &= test_case_6(&temp_dir, &port)?;

    cleanup_postgres(&temp_dir)?;

    if !all_tests_passed {
        std::process::exit(1);
    }
    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> io::Result<Output> {
    let output = Command::new(program)
        .args(args)
        .output()?;
    
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

fn find_available_port() -> io::Result<String> {
    TcpListener::bind("127.0.0.1:0")
        .map(|listener| listener.local_addr().unwrap().port().to_string())
}

fn setup_postgres(temp_dir: &Path, port: &str) -> io::Result<()> {
    let data_dir = temp_dir.join("data");
    fs::create_dir_all(&data_dir)?;
    let log_file_str = temp_dir.join("psql_test.log").to_string_lossy().into_owned();
    let data_dir_str = data_dir.to_string_lossy().into_owned();
    run_cmd("initdb", &["-D", &data_dir_str, "--set", &format!("port={}", port)])?;
    run_cmd("pg_ctl", &["start", "-D", &data_dir_str, "-l", &log_file_str])?;

    Ok(())
}

fn prepare_test_database(port: &str) -> io::Result<()> {
    run_cmd("createdb", &["-p", port, "psql_test"])?;
    run_cmd("psql", &["-p", port, "psql_test", "-c", "CREATE TABLE int8_tbl(q1 int8, q2 int8);"])?;

    Ok(())
}

fn generate_test_files(temp_dir: &Path, port: &str) -> io::Result<()> {
    run_cmd("psql", &["-p", port, "psql_test", "-c", "INSERT INTO int8_tbl VALUES (1,2), (3,4);"])?;
    let data_path = temp_dir.join("int8_tbl.data").to_string_lossy().into_owned();
    let csv_path = temp_dir.join("int8_tbl.csv").to_string_lossy().into_owned();
    let bin_path = temp_dir.join("int8_tbl.bin").to_string_lossy().into_owned();
    run_cmd("psql", &["-p", port, "psql_test", "-c", &format!("COPY int8_tbl TO '{}'", data_path)])?;
    run_cmd("psql", &["-p", port, "psql_test", "-c", &format!("COPY int8_tbl TO '{}' (format csv)", csv_path)])?;
    run_cmd("psql", &["-p", port, "psql_test", "-c", &format!("COPY int8_tbl TO '{}' (format binary)", bin_path)])?;
    let data_stdin_path = temp_dir.join("int8_tbl_data_stdin.sql");
    let csv_stdin_path = temp_dir.join("int8_tbl_csv_stdin.sql");
    let binary_stdin_path = temp_dir.join("int8_tbl_binary_stdin.sql");
    let mut file = File::create(&data_stdin_path)?;
    writeln!(file, "\\copy int8_tbl from stdin")?;
    let data_content = fs::read_to_string(&data_path)?;
    write!(file, "{}", data_content)?;
    let mut file = File::create(&csv_stdin_path)?;
    writeln!(file, "\\copy int8_tbl from stdin with (format csv)")?;
    let csv_content = fs::read_to_string(&csv_path)?;
    write!(file, "{}", csv_content)?;
    let mut file = File::create(&binary_stdin_path)?;
    writeln!(file, "\\copy int8_tbl from stdin with (format binary)")?;
    let binary_content = fs::read(&bin_path)?;
    file.write_all(&binary_content)?;

    Ok(())
}

fn test_case_1(temp_dir: &Path, port: &str) -> Result<bool, Box<dyn Error>> {
    println!("Running Test Case 1: \\copy int8_tbl from '/path/to/file' (text format)");
    truncate_table(port)?;
    let data_path = temp_dir.join("int8_tbl.data").to_string_lossy().into_owned();
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", &format!("\\copy int8_tbl from '{}'", data_path)])?;
    verify_copy_two!(output);
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", "SELECT * FROM int8_tbl;"])?;
    verify_result_set!(output);
    Ok(true)
}

fn test_case_2(temp_dir: &Path, port: &str) -> Result<bool, Box<dyn Error>> {
    println!("Running Test Case 2: psql -f file.sql containing '\\copy int8_tbl from stdin'");
    truncate_table(port)?;
    let stdin_path = temp_dir.join("int8_tbl_data_stdin.sql").to_string_lossy().into_owned();
    let output = run_cmd("psql", &["-p", port, "psql_test", "-f", &stdin_path])?;
    verify_copy_two!(output);
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", "SELECT * FROM int8_tbl;"])?;
    verify_result_set!(output);
    Ok(true)
}

fn test_case_3(temp_dir: &Path, port: &str) -> Result<bool, Box<dyn Error>> {
    println!("Running Test Case 3: \\copy int8_tbl from '/path/to/file' with (format binary)");
    truncate_table(port)?;
    let bin_path = temp_dir.join("int8_tbl.bin").to_string_lossy().into_owned();
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", &format!("\\copy int8_tbl from '{}' with (format binary)", bin_path)])?;
    verify_copy_two!(output);
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", "SELECT * FROM int8_tbl;"])?;
    verify_result_set!(output);
    Ok(true)
}

fn test_case_4(temp_dir: &Path, port: &str) -> Result<bool, Box<dyn Error>> {
    println!("Running Test Case 4: psql -f file.sql containing '\\copy int8_tbl from stdin with (format binary)'");
    truncate_table(port)?;
    let bin_stdin_path = temp_dir.join("int8_tbl_binary_stdin.sql").to_string_lossy().into_owned();
    let output = run_cmd("psql", &["-p", port, "psql_test", "-f", &bin_stdin_path])?;
    verify_copy_two!(output);
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", "SELECT * FROM int8_tbl;"])?;
    verify_result_set!(output);
    Ok(true)
}

fn test_case_5(_: &Path, port: &str) -> Result<bool, Box<dyn Error>> {
    println!("Running Test Case 5: \\copy int8_tbl from '/dev/tty' (interactive terminal input)");
    truncate_table(port)?;

    let temp_file = tempfile::NamedTempFile::new()?;
    let log_file = temp_file.as_file();
    
    let mut session = session::log(
        spawn(format!("psql -p {} psql_test", port))?,
        log_file.try_clone()?
    )?;

    session.set_expect_timeout(Some(Duration::from_secs(1)));

    expect!(&mut session, "psql_test=#", &temp_file);
    session.send_line("\\copy int8_tbl from '/dev/tty'")?;
    expect!(&mut session, "Enter data to be copied followed by a newline.", &temp_file);
    expect!(&mut session, "End with a backslash and a period on a line by itself, or an EOF signal.", &temp_file);
    expect!(&mut session, ">>", &temp_file);
    session.send_line("1\t2")?;
    expect!(&mut session, ">>", &temp_file);
    session.send_line("3\t4")?;
    expect!(&mut session, ">>", &temp_file);
    session.send_line("\\.")?;
    expect!(&mut session, ">>", &temp_file);
    write!(session, "\x04")?;
    expect!(&mut session, "psql_test=#", &temp_file);
    session.send_line("\\q")?;
    session.expect(Eof)?;
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", "SELECT * FROM int8_tbl;"])?;
    verify_result_set!(output);
    Ok(true)
}

fn test_case_6(_: &Path, port: &str) -> Result<bool, Box<dyn Error>> {
    println!("Running Test Case 6: \\copy int8_tbl from stdin (interactive prompt)");
    truncate_table(port)?;

    let temp_file = tempfile::NamedTempFile::new()?;
    let log_file = temp_file.as_file();
    
    let mut session = session::log(
        spawn(format!("psql -p {} psql_test", port))?,
        log_file.try_clone()?
    )?;

    session.set_expect_timeout(Some(Duration::from_secs(1)));

    expect!(&mut session, "psql_test=#", &temp_file);
    session.send_line("\\copy int8_tbl from stdin")?;
    expect!(&mut session, "Enter data to be copied followed by a newline.", &temp_file);
    expect!(&mut session, "End with a backslash and a period on a line by itself, or an EOF signal.", &temp_file);
    expect!(&mut session, ">>", &temp_file);
    session.send_line("1\t2")?;
    expect!(&mut session, ">>", &temp_file);
    session.send_line("3\t4")?;
    expect!(&mut session, ">>", &temp_file);
    session.send_line("\\.")?;
    expect!(&mut session, "COPY 2", &temp_file);
    expect!(&mut session, "psql_test=#", &temp_file);
    session.send_line("\\q")?;
    session.expect(Eof)?;
    
    let output = run_cmd("psql", &["-p", port, "psql_test", "-c", "SELECT * FROM int8_tbl;"])?;
    verify_result_set!(output);
    Ok(true)
}

fn truncate_table(port: &str) -> io::Result<()> {
    run_cmd("psql", &["-p", port, "psql_test", "-c", "TRUNCATE int8_tbl;"])?;
    Ok(())
}

fn cleanup_postgres(temp_dir: &Path) -> io::Result<()> {
    let data_dir = temp_dir.join("data");
    run_cmd("pg_ctl", &["stop", "-D", data_dir.to_str().unwrap(), "-m", "immediate"])?;
    fs::remove_dir_all(temp_dir)?;
    Ok(())
}
