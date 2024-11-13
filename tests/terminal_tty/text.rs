use crate::common::*;
use expectrl::{session, spawn, Eof};
use similar::{ChangeTag, TextDiff};
use std::borrow::Cow;
use std::error::Error;
use std::io::Write;
use std::time::Duration;
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
    session.send_line(&format!(r#"\copy "{}" from '/dev/tty'"#, test_table))?;
    expect!(&mut session, "Enter data to be copied followed by a newline.", &temp_file);
    expect!(&mut session, "End with an EOF signal.", &temp_file);
    expect!(&mut session, ">>", &temp_file);
    session.send_line("1\t2")?;
    expect!(&mut session, ">>", &temp_file);
    session.send_line("3\t4")?;
    expect!(&mut session, ">>", &temp_file);
    session.send_line("\\.")?;
    expect!(&mut session, ">>", &temp_file);
    write!(session, "\x04")?;
    expect!(&mut session, &format!("{}=#", database_name), &temp_file);
    session.send_line("\\q")?;
    session.expect(Eof)?;

    let output = run_cmd("psql", &["-c", &format!(r#"SELECT * FROM "{}";"#, test_table)])?;
    expect_result_set!(output);
    let output = run_cmd("psql", &["-c", &format!(r#"DROP TABLE "{}";"#, test_table)])?;
    expect_drop_table!(output);
    Ok(())
}