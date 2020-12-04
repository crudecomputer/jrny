//! Utilities for parsing SQL scripts into individual statements.
//! There is no intention to perform any validation or statement preparation
//! in the database; the primary use case is mainly better timing, logging,
//! and user feedback.
use std::convert::TryFrom;
use std::slice::Iter;

use crate::parser::Parser;

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
    type Error = String;
    
    /// Attempts to parse the input into individual statements.
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let statements = Parser::parse(input);

        // Transaction-management commands should cause immediate errors,
        // and thankfully it's just exact keyword matching at the start
        // (provided the string is TRIMMED) and it doesn't matter if
        // they're embedded inside a string or delimited identifier at all.
        for s in &statements {
            let lowered = s.0.chars()
                .take(10)
                .collect::<String>()
                .to_lowercase();

            for command in ["begin", "savepoint", "rollback", "commit"].iter() {
                if lowered.starts_with(command) {
                    return Err(format!(
                        "{} command is not supported in a revision",
                        command.to_uppercase(),
                    ));
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
            "#).unwrap(),
            StatementGroup(vec![
                Statement(
r#"create table "some;thing"  (
    id serial primary key, 
    name text not null
)"#
                .to_string()),
                Statement(
r#"insert into "some;thing" (name) 
    values ('is this;a thing?')"#
                .to_string()),
            ])
        );
    }

    #[test]
    fn test_errors_from_transaction_commands() {
        let err = |cmd| Err(format!(
            "{} command is not supported in a revision",
            cmd,
        ));

        assert_eq!(StatementGroup::try_from(" beGIN "),         err("BEGIN"));
        assert_eq!(StatementGroup::try_from("one; begin; two"), err("BEGIN"));
        assert_eq!(StatementGroup::try_from("ONE; BEGIN; TWO"), err("BEGIN"));

        assert_eq!(StatementGroup::try_from("  savEPOint "),        err("SAVEPOINT"));
        assert_eq!(StatementGroup::try_from("one; savepoint; two"), err("SAVEPOINT"));
        assert_eq!(StatementGroup::try_from("ONE; SAVEPOINT; TWO"), err("SAVEPOINT"));

        assert_eq!(StatementGroup::try_from("  rOLLBack "),        err("ROLLBACK"));
        assert_eq!(StatementGroup::try_from("one; rollback; two"), err("ROLLBACK"));
        assert_eq!(StatementGroup::try_from("ONE; ROLLBACK; TWO"), err("ROLLBACK"));

        assert_eq!(StatementGroup::try_from("  coMMIt "),        err("COMMIT"));
        assert_eq!(StatementGroup::try_from("one; commit; two"), err("COMMIT"));
        assert_eq!(StatementGroup::try_from("ONE; COMMIT; TWO"), err("COMMIT"));

        assert_eq!(StatementGroup::try_from("begin; rollback; savepoint; commit"), err("BEGIN"));
        assert_eq!(StatementGroup::try_from("rollback; begin; savepoint; commit"), err("ROLLBACK"));
        assert_eq!(StatementGroup::try_from("savepoint; begin; rollback; commit"), err("SAVEPOINT"));
        assert_eq!(StatementGroup::try_from("commit; begin; rollback; commit"),    err("COMMIT"));
    }
}
