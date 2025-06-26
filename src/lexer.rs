// src/lexer.rs

use logos::Logos;

/// A token produced by the TeX lexer.
#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    /// Skip comments starting with `%` up to end of line.
    #[regex(r"%[^\n]*", logos::skip)]
    Comment,

    /// Left brace `{`.
    #[token("{")]
    LBrace,

    /// Right brace `}`.
    #[token("}")]
    RBrace,

    /// TeX command starting with backslash, e.g. `\textbf`.
    #[regex(r"\\[a-zA-Z]+", callback = |lex| lex.slice()[1..].to_string(), priority = 2)]
    Command(String),

    /// Drop all whitespace: spaces, tabs, newlines
    #[regex(r"\s+", logos::skip)]
    Whitespace,

    #[regex(r"//[^\n]*", logos::skip)]
    CppComment,

    /// Any sequence of characters not including `\\`, `{`, `}`, whitespace, or `%`.
    #[regex(r"[^\\{}\s%]+", callback = |lex| lex.slice().to_string(), priority = 1)]
    Text(String),

    /// Catch any unrecognized character.
    #[error]
    Error,
}

/// A spanned token: the token plus its start and end byte offsets in the input.
pub type SpannedToken = (Token, usize, usize);

/// Lex the input TeX string into a vector of spanned tokens.
pub fn lex(input: &str) -> Vec<SpannedToken> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(input);
    while let Some(tok) = lexer.next() {
        let span = lexer.span();
        tokens.push((tok, span.start, span.end));
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get just the token kinds from lexed output.
    fn kinds(input: &str) -> Vec<Token> {
        lex(input).into_iter().map(|(t, _, _)| t).collect()
    }

    #[test]
    fn test_text() {
        assert_eq!(kinds("Hello"), vec![Token::Text("Hello".into())]);
    }

    #[test]
    fn test_command() {
        assert_eq!(kinds("\\textbf"), vec![Token::Command("textbf".into())]);
    }

    #[test]
    fn test_braces() {
        assert_eq!(kinds("{ }"), vec![Token::LBrace, Token::RBrace]);
    }

    #[test]
    fn test_mixed() {
        let input = "\\emph{Word} and text";
        let expected = vec![
            Token::Command("emph".into()),
            Token::LBrace,
            Token::Text("Word".into()),
            Token::RBrace,
            Token::Text("and".into()),
            Token::Text("text".into()),
        ];
        assert_eq!(kinds(input), expected);
    }

    #[test]
    fn test_comment() {
        // Text before comment, comment skipped, then More
        assert_eq!(
            kinds("Text % comment\nMore"),
            vec![Token::Text("Text".into()), Token::Text("More".into()),]
        );
    }
}
