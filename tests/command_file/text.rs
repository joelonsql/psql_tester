use crate::common::*;
use similar::{ChangeTag, TextDiff};
use std::borrow::Cow;
use std::error::Error;
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;

#[test]
fn test_psql_copy() -> Result<(), Box<dyn Error>> {
    let env = get_test_environment();
    let test_table = Uuid::new_v4();
    let output = run_cmd("psql", &["-c", &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table)])?;
    expect_create_table!(output);
    let output = run_cmd("psql", &["-c", &format!(r#"\copy "{}" from '{}'"#, test_table, env.file_path_text)])?;
    expect_copy_two!(output);
    let output = run_cmd("psql", &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)])?;
    expect_result_set!(output);
    let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
    expect_drop_table!(output);
    Ok(())
}