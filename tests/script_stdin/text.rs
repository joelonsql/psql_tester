use crate::common::*;
use similar::{ChangeTag, TextDiff};
use std::borrow::Cow;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;

#[test]
fn test_psql_copy() -> Result<(), Box<dyn Error>> {
    let env = get_test_environment();
    let test_table = Uuid::new_v4();
    let output = run_cmd("psql", &["-c", &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table)])?;
    expect_create_table!(output);
    let test_file_path = env.temp_dir.join(format!("{}", test_table));
    let mut test_file = File::create(&test_file_path)?;
    writeln!(test_file, r#"\copy "{}" from stdin"#, test_table)?;
    let data_content = fs::read_to_string(&env.file_path_text)?;
    write!(test_file, "{}", data_content)?;
    let output = run_cmd("psql", &["-f", &test_file_path.to_string_lossy().into_owned()])?;
    expect_copy_two!(output);
    let output = run_cmd("psql", &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)])?;
    expect_result_set!(output);
    let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
    expect_drop_table!(output);
    Ok(())
}