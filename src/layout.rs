use crate::ast::{Node, TextStyle};

/// A run of text with an associated style.
#[derive(Debug, Clone)]
pub struct StyledRun {
    pub text: String,
    pub style: TextStyle,
}

/// One primitive in the layout: either a styled run or flexible glue.
#[derive(Debug, Clone)]
pub enum LayoutNode {
    Run(StyledRun),
    Glue(f64),
}

/// A horizontal box of layout primitives, with a precomputed width.
#[derive(Debug, Clone)]
pub struct HBox {
    pub items: Vec<LayoutNode>,
    pub width: f64,
}

/// A single line: a sequence of HBoxes and its total width.
#[derive(Debug, Clone)]
pub struct Line {
    pub boxes: Vec<HBox>,
    pub width: f64,
}

/// A page: a sequence of lines.
#[derive(Debug, Clone)]
pub struct Page {
    pub lines: Vec<Line>,
}

/// Layout the AST into pages using a simple greedy line-breaker.
///
/// - `ast`: the expanded AST to lay out  
/// - `line_width`: maximum width per line, in the same units as `char_width`/`space_width`  
/// - `line_height`: assumed fixed line height (used only to decide how many lines per page)  
/// - `char_width`: advance width for each character  
/// - `space_width`: width of the glue inserted between words or nodes  
pub fn layout(
    ast: &Node,
    line_width: f64,
    line_height: f64,
    char_width: f64,
    space_width: f64,
) -> Vec<Page> {
    // 1) Flatten AST → HBoxes
    let mut hboxes = Vec::new();
    flatten_ast(ast, &mut hboxes, char_width, space_width);

    // 2) Greedy line-breaking
    let mut lines = Vec::new();
    let mut curr = Vec::new();
    let mut w = 0.0;
    for hb in hboxes {
        if w + hb.width > line_width && !curr.is_empty() {
            lines.push(Line {
                boxes: curr,
                width: w,
            });
            curr = Vec::new();
            w = 0.0;
        }
        w += hb.width;
        curr.push(hb);
    }
    if !curr.is_empty() {
        lines.push(Line {
            boxes: curr,
            width: w,
        });
    }

    // 3) Fixed page-breaking
    let mut pages = Vec::new();
    let max_lines = (800.0 / line_height).floor() as usize;
    for chunk in lines.chunks(max_lines) {
        pages.push(Page {
            lines: chunk.to_vec(),
        });
    }
    pages
}

/// Recursively flatten the AST into HBoxes.
///
/// Strategy:
/// 1. Split each Text/StyledText on internal whitespace → words + inter-word glue.
/// 2. Inject one `space_width` glue between every two AST siblings (Seq or Macro).
fn flatten_ast(ast: &Node, hboxes: &mut Vec<HBox>, char_width: f64, space_width: f64) {
    // A small helper to build a glue-box of the given width:
    let glue = || HBox {
        items: vec![LayoutNode::Glue(space_width)],
        width: space_width,
    };

    match ast {
        Node::Seq(children) => {
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    hboxes.push(glue());
                }
                flatten_ast(child, hboxes, char_width, space_width);
            }
        }

        Node::Text(s) => {
            // split on any run of whitespace → words
            for (j, word) in s.split_whitespace().enumerate() {
                if j > 0 {
                    hboxes.push(glue());
                }
                let run = StyledRun {
                    text: word.to_string(),
                    style: TextStyle::Normal,
                };
                let width = word.len() as f64 * char_width;
                hboxes.push(HBox {
                    items: vec![LayoutNode::Run(run)],
                    width,
                });
            }
        }

        Node::StyledText(s, style) => {
            for (j, word) in s.split_whitespace().enumerate() {
                if j > 0 {
                    hboxes.push(glue());
                }
                let run = StyledRun {
                    text: word.to_string(),
                    style: style.clone(),
                };
                let width = word.len() as f64 * char_width;
                hboxes.push(HBox {
                    items: vec![LayoutNode::Run(run)],
                    width,
                });
            }
        }

        Node::Macro { name: _, args } => {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    hboxes.push(glue());
                }
                flatten_ast(arg, hboxes, char_width, space_width);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Node, TextStyle};

    #[test]
    fn test_layout_simple() {
        let ast = Node::Seq(vec![Node::Text("Hello world".into())]);
        // line_width=100, line_height=20, char=6, space=6
        let pages = layout(&ast, 100.0, 20.0, 6.0, 6.0);
        assert_eq!(pages.len(), 1);
        let line = &pages[0].lines[0];
        // expect: "Hello" glue "world" ⇒ 3 boxes
        assert_eq!(line.boxes.len(), 3);
    }

    #[test]
    fn test_line_break() {
        let long = "ThisIsAVeryLongWordWithoutSpaces";
        let ast = Node::Seq(vec![Node::Text(long.into())]);
        let pages = layout(&ast, 50.0, 20.0, 6.0, 6.0);
        // too narrow => a single HBox per line
        assert_eq!(pages[0].lines[0].boxes.len(), 1);
    }

    #[test]
    fn test_layout_styled() {
        let ast = Node::Seq(vec![
            Node::StyledText("Bold".into(), TextStyle::Bold),
            Node::Text("and normal".into()),
        ]);
        let pages = layout(&ast, 100.0, 20.0, 6.0, 6.0);
        let line = &pages[0].lines[0];
        // Bold | space | "and" | space | "normal" => 5 boxes
        assert_eq!(line.boxes.len(), 5);
    }
}
