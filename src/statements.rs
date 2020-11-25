// read a revision
// parse into list of commands
// error if encountering begin; rollback; commit; savepoint;

use std::mem;

#[derive(Debug, Default, PartialEq)]
struct Statement(String);

#[derive(Debug)]
struct StatementGroup {
    statements: Vec<Statement>,
}

impl StatementGroup {
    fn new(text: &str) -> Result<Self, String> {
        Ok(Self {
            statements: parse(text)?,
        })
    }
}

fn parse(text: &str) -> Result<Vec<Statement>, String> {
    let mut parser = Parser::new();

    for c in text.trim().chars() {
        parser.accept(c);
    }

    Ok(parser.statements
        .drain(..)
        .filter(|stmt| !stmt.0.is_empty())
        .map(|stmt| Statement(stmt.0.trim().to_string()))
        .collect())
}

struct Parser {
    statements: Vec<Statement>,
    in_string: bool,
    in_delimited_identifier: bool,
}

impl Parser {
    fn new() -> Self {
        Self {
            statements: vec![],
            in_string: false,
            in_delimited_identifier: false,
        }
    }

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
        // A semicolon ending a statement is the only time the character
        // SHOULD NOT be appended; instead, it should "end" the current
        // statement by creating a new one.
        if c == ';' && !self.in_string && !self.in_delimited_identifier {
            self.statements.push(Statement::default());

            return;
        }

        if self.statements.len() == 0 {
            self.statements.push(Statement::default());
        }

        self.statements
            .last_mut()
            .unwrap()
            .0.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let expected: Vec<Statement> = vec![];

        assert_eq!(parse("").unwrap(), expected);
        assert_eq!(parse("  ").unwrap(), expected);
        assert_eq!(parse("  \n  \n  ").unwrap(), expected);
    }

    #[test]
    fn test_single() {
        assert_eq!(
            parse("anything really, does not matter").unwrap(),
            vec![
                Statement("anything really, does not matter".to_string()),
            ],
        );
    }

    #[test]
    fn test_single_with_embedded_semicolons() {
        assert_eq!(
            parse("one thing ';' and two things \";\"").unwrap(),
            vec![
                Statement("one thing ';' and two things \";\"".to_string()),
            ],
        );
    }

    #[test]
    fn test_multiple_without_embedded() {
        assert_eq!(
            parse("  one thing  ; two things ").unwrap(),
            vec![
                Statement("one thing".to_string()),
                Statement("two things".to_string()),
            ],
        );
    }
}