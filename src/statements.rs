//! Utilities for parsing SQL scripts into individual statements.
//! There is no intention to perform any validation or statement preparation
//! in the database; the primary use case is mainly better timing, logging,
//! and user feedback.
use std::convert::TryFrom;
use std::slice::Iter;

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
        let mut parser = Parser::default();

        // Strip any lines that 
        let without_comments: String = input.lines()
            .filter(|l| !l.trim().starts_with("--"))
            .fold(String::new(), |a, b| a + b + "\n");

        for c in without_comments.chars() {
            parser.accept(c);
        }

        // If the parser handled white-space better, the extra allocations
        // here would not be necessary... TODO
        let statements: Vec<Statement> = parser.statements.iter()
            .map(|stmt| Statement(stmt.0.trim().to_string()))
            .filter(|stmt| !stmt.0.is_empty())
            .collect();

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

/// A simple pseudo-state machine that generates a vec of individual statements
/// by accepting one character at a time.
#[derive(Default)]
struct Parser {
    statements: Vec<Statement>,
    in_string: bool,
    in_delimited_identifier: bool,
}

impl Parser {
    /// Appends the char to the current statement, ignore the character, or begins
    /// a new statement depending on the given char.
    fn accept(&mut self, c: char) {
        // A single quote can open or close a text string, but ONLY if
        // it's not embedded in a delimited identifier
        if c == '\'' && !self.in_delimited_identifier {
            self.in_string = !self.in_string;
        }

        // Likewise, a double quote can open or close a delimited identifer,
        // but only if it's not inside a text string
        if c == '"' && !self.in_string {
            self.in_delimited_identifier = !self.in_delimited_identifier;
        }

        // Meanwhile, back at the ranch, a semicolon ends a statement
        // only if it's outside of text strings or quoted identifiers.
        // It doesn't need to be appended; it only needs to end the
        // "current" statement by creating a new one.
        if c == ';' && !self.in_string && !self.in_delimited_identifier {
            self.statements.push(Statement::default());

            return;
        }

        if self.statements.len() == 0 {
            self.statements.push(Statement::default());
        }

        // `unwrap` is safe here, as this is guaranteed to have an element
        self.statements.last_mut().unwrap().0.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let empty = StatementGroup(vec![]);

        assert_eq!(StatementGroup::try_from("").unwrap(), empty);
        assert_eq!(StatementGroup::try_from("  ").unwrap(), empty);
        assert_eq!(StatementGroup::try_from("  \n  \n  ").unwrap(), empty);
        assert_eq!(StatementGroup::try_from(" ;; ; ;  ;").unwrap(), empty);
    }

    #[test]
    fn test_single() {
        assert_eq!(
            StatementGroup::try_from("anything really, does not matter").unwrap(),
            StatementGroup(vec![
                Statement("anything really, does not matter".to_string()),
            ]),
        );
    }

    #[test]
    fn test_single_with_embedded_semicolons() {
        assert_eq!(
            StatementGroup::try_from("one thing ';' and two things \";\"").unwrap(),
            StatementGroup(vec![
                Statement("one thing ';' and two things \";\"".to_string()),
            ]),
        );
    }

    #[test]
    fn test_multiple_without_embedded() {
        assert_eq!(
            StatementGroup::try_from("  one thing  ; two things ").unwrap(),
            StatementGroup(vec![
                Statement("one thing".to_string()),
                Statement("two things".to_string()),
            ]),
        );
    }

    #[test]
    fn test_quoted_with_semicolons() {
        assert_eq!(
            StatementGroup::try_from(r#" '";'"  "#).unwrap(),
            StatementGroup(vec![
                Statement(r#"'";'""#.to_string()),
            ]),
        );
        assert_eq!(
            StatementGroup::try_from(r#" '"';"  "#).unwrap(),
            StatementGroup(vec![
                Statement(r#"'"'"#.to_string()),
                Statement(r#"""#.to_string()),
            ]),
        );
        assert_eq!(
            StatementGroup::try_from(r#" a ';' b ";" c '";"' d "';'" e    "#).unwrap(),
            StatementGroup(vec![
                Statement(r#"a ';' b ";" c '";"' d "';'" e"#.to_string()),
            ]),
        );
    }

    #[test]
    fn test_inline_comments_with_semicolon_on_own_line() {
        assert_eq!(
            StatementGroup::try_from("
                this is one statement;
                this is
                -- ;
                another statement
            ").unwrap(),
            StatementGroup(vec![
                Statement("this is one statement".to_string()),
                Statement("this is another statement".to_string()),
            ]),
        );
    }

    #[test]
    fn test_inline_comments_with_semicolon_trailing() {
        assert_eq!(
            StatementGroup::try_from("
                this is one statement;
                this is -- ;
                another statement
            ").unwrap(),
            StatementGroup(vec![
                Statement("this is one statement".to_string()),
                Statement("this is another statement".to_string()),
            ]),
        );
    }

    #[test]
    fn test_block_comments_with_semicolons() {
        // own lines
        // inline
        assert_eq!(true, false);
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
