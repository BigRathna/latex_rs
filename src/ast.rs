#[derive(Clone, Debug, PartialEq)]
pub enum TextStyle {
    Normal,
    Bold,
    Italic,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Text(String),
    StyledText(String, TextStyle),
    // Whitespace(usize),
    Macro { name: String, args: Vec<Node> },
    Seq(Vec<Node>),
}

impl Node {
    /// Convenience constructor for a text node.
    pub fn text<T: Into<String>>(s: T) -> Self {
        Node::Text(s.into())
    }

    /// Convenience constructor for a macro node.
    pub fn r#macro<T: Into<String>>(name: T, args: Vec<Node>) -> Self {
        Node::Macro {
            name: name.into(),
            args,
        }
    }

    /// Convenience constructor for a sequence node.
    pub fn seq(children: Vec<Node>) -> Self {
        Node::Seq(children)
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn test_node_eq() {
        let a = Node::text("Hello");
        let b = Node::Text("Hello".into());
        assert_eq!(a, b);
    }

    #[test]
    fn test_macro_node() {
        let m = Node::r#macro("test", vec![Node::text("arg")]);
        if let Node::Macro { name, args } = m {
            assert_eq!(name, "test");
            assert_eq!(args, vec![Node::Text("arg".into())]);
        } else {
            panic!("Expected Macro variant");
        }
    }

    #[test]
    fn test_sequence() {
        let seq = Node::seq(vec![Node::text("A"), Node::text("B")]);
        assert_eq!(
            seq,
            Node::Seq(vec![Node::Text("A".into()), Node::Text("B".into())])
        );
    }
}
