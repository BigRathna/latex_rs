use crate::ast::{Node, TextStyle};
/// Perform macro-expansion on the AST for MVP.
/// Currently we don't support user-defined macros (\newcommand),
/// so this pass simply:
/// 1. Flattens nested sequences (Seq within Seq).
/// 2. Recurses into macro arguments for built-ins (e.g. textbf) to prepare for layout.

fn collect_plain_text(node: &Node) -> String {
    match node {
        Node::Text(s) | Node::StyledText(s, _) => s.clone(),
        Node::Seq(children) => children
            .iter()
            .map(collect_plain_text)
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(), // or panic!("Unexpected node in textbf")
    }
}

/// Expand macros in the AST (identity + flattening pass).
pub fn expand_macros(ast: &Node) -> Node {
    match ast {
        // For a sequence, recursively expand children and flatten nested Seq.
        Node::Seq(children) => {
            let mut flat_children = Vec::new();
            for child in children {
                let exp = expand_macros(child);
                match exp {
                    Node::Seq(inner) => flat_children.extend(inner),
                    other => flat_children.push(other),
                }
            }
            Node::Seq(flat_children)
        }

        // Whitespace nodes are unchanged; they'll become glue later.
        // Node::Whitespace(n) => Node::Whitespace(*n),

        // For macro invocations, expand their arguments.
        // We leave the Macro node intact so layout/backends can interpret built-in commands
        Node::Macro { name, args } => {
            // First, expand children of the macro
            let expanded_args = args.iter().map(expand_macros).collect::<Vec<_>>();

            // If it’s a styling macro with exactly one argument that is text (or Seq of Text),
            // collapse it into a StyledText node:
            if name == "textbf" && expanded_args.len() == 1 {
                // Flatten that one argument into a single String
                let content = collect_plain_text(&expanded_args[0]);
                return Node::StyledText(content, TextStyle::Bold);
            }
            if name == "emph" && expanded_args.len() == 1 {
                let content = collect_plain_text(&expanded_args[0]);
                return Node::StyledText(content, TextStyle::Italic);
            }

            // Otherwise, flatten Seq arguments as before
            let mut flat_args = Vec::new();
            for arg in expanded_args {
                match arg {
                    Node::Seq(inner) => flat_args.extend(inner),
                    other => flat_args.push(other),
                }
            }
            Node::Macro {
                name: name.clone(),
                args: flat_args,
            }
        }

        Node::Text(s) => Node::Text(s.clone()),

        Node::StyledText(s, style) => Node::StyledText(s.clone(), style.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;

    #[test]
    fn flattens_sequence() {
        let nested = Node::Seq(vec![
            Node::Text("A".into()),
            Node::Seq(vec![Node::Text("B".into()), Node::Text("C".into())]),
            Node::Text("D".into()),
        ]);
        let expected = Node::Seq(vec![
            Node::Text("A".into()),
            Node::Text("B".into()),
            Node::Text("C".into()),
            Node::Text("D".into()),
        ]);
        assert_eq!(expand_macros(&nested), expected);
    }

    #[test]
    fn recurses_into_macros() {
        let mac = Node::Macro {
            name: "cmd".into(),
            args: vec![Node::Seq(vec![Node::Text("X".into())])],
        };
        let expanded = expand_macros(&mac);
        // Should flatten the inner Seq in args but keep top-level Macro
        match expanded {
            Node::Macro { name, args } => {
                assert_eq!(name, "cmd");
                assert_eq!(args, vec![Node::Text("X".into())]);
            }
            _ => panic!("Expected Macro node"),
        }
    }
}
