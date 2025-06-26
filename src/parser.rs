use crate::ast::Node;
use crate::lexer::{Token, lex};

pub fn parse(input: &str) -> Result<Node, String> {
    let toks = lex(input)
        .into_iter()
        .map(|(t, _, _)| t)
        .collect::<Vec<_>>();
    let (ast, pos) = parse_sequence(&toks, 0)?;
    if pos != toks.len() {
        Err(format!("Unexpected tokens remaining at pos {}", pos))
    } else {
        Ok(ast)
    }
}

fn parse_sequence(tokens: &[Token], mut pos: usize) -> Result<(Node, usize), String> {
    let mut children = Vec::new();
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::RBrace => break,
            _ => {
                let (node, new_pos) = parse_node(tokens, pos)?;
                children.push(node);
                pos = new_pos;
            }
        }
    }
    Ok((Node::Seq(children), pos))
}

fn parse_node(tokens: &[Token], pos: usize) -> Result<(Node, usize), String> {
    match &tokens[pos] {
        // Token::Whitespace(n) => {
        //     // emit a Whitespace node, consuming 1 token
        //     Ok((Node::Whitespace(*n), pos + 1))
        // }
        Token::Text(text) => Ok((Node::Text(text.clone()), pos + 1)),
        Token::Command(name) => {
            // Expect a brace-argument next
            let mut args = Vec::new();
            let mut cur = pos + 1;
            if cur < tokens.len() && tokens[cur] == Token::LBrace {
                // parse group as single arg
                let (arg_node, new_pos) = parse_group(tokens, cur)?;
                args.push(arg_node);
                cur = new_pos;
            }
            Ok((
                Node::Macro {
                    name: name.clone(),
                    args,
                },
                cur,
            ))
        }
        Token::LBrace => parse_group(tokens, pos),
        unexpected => Err(format!("Unexpected token at {}: {:?}", pos, unexpected)),
    }
}

fn parse_group(tokens: &[Token], pos: usize) -> Result<(Node, usize), String> {
    // tokens[pos] should be LBrace
    if tokens[pos] != Token::LBrace {
        return Err(format!("Expected '{{' at {}, found {:?}", pos, tokens[pos]));
    }
    // Parse inner sequence starting after LBrace
    let (inner, mut cur) = parse_sequence(tokens, pos + 1)?;
    // Expect RBrace
    if cur < tokens.len() && tokens[cur] == Token::RBrace {
        Ok((inner, cur + 1))
    } else {
        Err(format!("Unclosed '{{' at {}", pos))
    }
}

#[cfg(test)]
mod tests {
    // Bring your parse() function into scope:
    use crate::parser::parse;
    // Bring your AST Node enum into scope:
    use crate::ast::Node;

    #[test]
    fn test_simple_text() {
        let ast = parse("Hello").unwrap();
        assert_eq!(ast, Node::Seq(vec![Node::Text("Hello".into())]));
    }

    #[test]
    fn test_grouping() {
        let ast = parse("{Hi there}").unwrap();
        assert_eq!(
            ast,
            Node::Seq(vec![Node::Seq(vec![
                Node::Text("Hi".into()),
                Node::Text("there".into()),
            ])])
        );
    }

    #[test]
    fn test_macro() {
        let ast = parse(r"\textbf{Bold}").unwrap();
        assert_eq!(
            ast,
            Node::Seq(vec![Node::Macro {
                name: "textbf".into(),
                args: vec![Node::Seq(vec![Node::Text("Bold".into())])],
            }])
        );
    }
}
