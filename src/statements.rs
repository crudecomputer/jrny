//! Utilities for parsing SQL scripts into individual statements.
//! There is no intention to perform any validation or statement preparation
//! in the database; the primary use case is mainly better timing, logging,
//! and user feedback.
use std::convert::TryFrom;
use std::slice::Iter;

use crate::{Error, parser::Parser};

/// An individual raw SQL statement.
#[derive(Debug, Default, PartialEq)]
pub struct Statement(pub String);

/// A group of raw SQL statements from a single file.
#[derive(Debug, PartialEq)]
pub struct StatementGroup(Vec<Statement>);

impl StatementGroup {
    pub fn iter(&self) -> Iter<Statement> {
        self.0.iter()
    }
}

impl TryFrom<&str> for StatementGroup {
    type Error = crate::Error;

    /// Attempts to parse the input into individual statements.
    fn try_from(input: &str) -> std::result::Result<Self, Self::Error> {
        let statements = Parser::parse(input);

        // Transaction-management commands should cause immediate errors,
        // and thankfully it's just exact keyword matching at the start
        // (provided the string is TRIMMED) and it doesn't matter if
        // they're embedded inside a string or delimited identifier at all.
        for s in &statements {
            let lowered = s.0.chars().take(10).collect::<String>().to_lowercase();

            for command in ["begin", "savepoint", "rollback", "commit"].iter() {
                if lowered.starts_with(command) {
                    return Err(Error::TransactionCommandFound(command.to_string()));
                }
            }
        }

        Ok(Self(statements))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parses_multiple_statements() {
        assert_eq!(
            StatementGroup::try_from(
                r#"create table "some;thing" /*
    **NOTE** the various comments and semicolons
*/ (
    id serial primary key, -- oh; yeah
    name text not null
);

insert into "some;thing" (name) -- here we go;
    values ('is this;a thing?');

;
-- hmmm;;
;
            "#
            )
            .unwrap(),
            StatementGroup(vec![
                Statement(
                    r#"create table "some;thing"  (
    id serial primary key, 
    name text not null
)"#
                    .to_string()
                ),
                Statement(
                    r#"insert into "some;thing" (name) 
    values ('is this;a thing?')"#
                        .to_string()
                ),
            ])
        );
    }

    #[test]
    fn test_errors_from_transaction_commands() {
        let is_err = |statement, cmd| match StatementGroup::try_from(statement) {
            Err(Error::TransactionCommandFound(c)) if c == cmd => {},
            result => panic!("received {:?}", result),
        };

        is_err(" beGIN ",         "begin");
        is_err("one; begin; two", "begin");
        is_err("ONE; BEGIN; TWO", "begin");

        is_err("  savEPOint ",        "savepoint");
        is_err("one; savepoint; two", "savepoint");
        is_err("ONE; SAVEPOINT; TWO", "savepoint");

        is_err("  rOLLBack ",        "rollback");
        is_err("one; rollback; two", "rollback");
        is_err("ONE; ROLLBACK; TWO", "rollback");

        is_err("  coMMIt ",        "commit");
        is_err("one; commit; two", "commit");
        is_err("ONE; COMMIT; TWO", "commit");

        is_err("begin; rollback; savepoint; commit", "begin");
        is_err("rollback; begin; savepoint; commit", "rollback");
        is_err("savepoint; begin; rollback; commit", "savepoint");
        is_err("commit; begin; rollback; commit",    "commit");
    }
}
