// read a revision

// parse into list of commands

// error if encountering begin; rollback; commit; savepoint;
#[derive(Debug, PartialEq)]
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
    Ok(text.trim()
        .split(";")
        .filter(|sub| !sub.is_empty())
        .map(|sub| Statement(sub.to_string()))
        .collect())
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
        )
    }

    #[test]
    fn test_single_with_semicolon() {
        
    }
}