use crate::common::*;
use expectrl::{session, spawn};
use std::error::Error;
use std::io::Write;
use std::time::Duration;
use std::borrow::Cow;
use similar::{ChangeTag, TextDiff};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;

#[test]
fn test_psql_copy() -> Result<(), Box<dyn Error>> {
    let test_table = Uuid::new_v4();
    let output = run_cmd("psql", &["-c", &format!(r#"CREATE TABLE "{}" (c1 int8, c2 int8);"#, test_table)])?;
    expect_create_table!(output);

    let temp_file = tempfile::NamedTempFile::new()?;
    let log_file = temp_file.as_file();

    let mut session = session::log(spawn("psql")?, log_file.try_clone()?)?;

    session.set_expect_timeout(Some(Duration::from_secs(1)));

    let database_name =
        std::env::var("PGDATABASE").unwrap_or_else(|_| std::env::var("USER").unwrap());

    expect!(&mut session, &format!("{}=#", database_name), &temp_file);
    session.send_line(&format!(r#"\copy "{}" from '/dev/tty' (format binary)"#, test_table))?;
    expect!(&mut session, "End with an EOF signal.", &temp_file);

    // XXX - Sending the actual binary data is untested, but is it even possible?
    Ok(())
}