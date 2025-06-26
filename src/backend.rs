use crate::ast::TextStyle;
use crate::layout::{LayoutNode, Page};
use printpdf::{Mm, PdfDocument};
use std::io::{BufWriter, Cursor};

/// Render a sequence of laid-out pages into a PDF document.
///
/// # Parameters
/// - `pages`: the output of your `layout()` function.
///
/// # Returns
/// A `Vec<u8>` containing the PDF file.
pub fn render_pdf(pages: &[Page]) -> Vec<u8> {
    // 1) Create a new A4 PDF (210×297 mm)
    let (doc, first_page, first_layer) =
        PdfDocument::new("latex_rs_document", Mm(210.0), Mm(297.0), "Layer 1");

    // 2) Embed your three font faces
    let font_normal = doc
        .add_external_font(Cursor::new(
            include_bytes!("../fonts/DejaVuSans.ttf") as &[u8]
        ))
        .unwrap();
    let font_bold = doc
        .add_external_font(Cursor::new(
            include_bytes!("../fonts/DejaVuSans-Bold.ttf") as &[u8]
        ))
        .unwrap();
    let font_italic = doc
        .add_external_font(Cursor::new(
            include_bytes!("../fonts/DejaVuSans-Oblique.ttf") as &[u8],
        ))
        .unwrap();

    // 3) Metrics & margins
    let font_size_pt = 12.0;
    let line_height_mm = Mm(font_size_pt * 1.2 * 25.4 / 72.0);
    let margin_left = Mm(10.0);
    let margin_top = Mm(10.0);

    // 4) Draw each page
    for (idx, page) in pages.iter().enumerate() {
        // Get the right page & layer
        let (page_idx, layer_idx) = if idx == 0 {
            (first_page, first_layer)
        } else {
            doc.add_page(Mm(210.0), Mm(297.0), format!("Layer {}", idx + 1))
        };
        let layer = doc.get_page(page_idx).get_layer(layer_idx);

        // Start at top-left (printing origin is bottom-left in PDF)
        let mut current_y = Mm(297.0) - margin_top;

        for line in &page.lines {
            let mut current_x = margin_left;

            for hbox in &line.boxes {
                for node in &hbox.items {
                    match node {
                        // Styled text runs
                        LayoutNode::Run(run) => {
                            let font = match run.style {
                                TextStyle::Normal => &font_normal,
                                TextStyle::Bold => &font_bold,
                                TextStyle::Italic => &font_italic,
                            };
                            // Emit the text
                            layer.use_text(&run.text, font_size_pt, current_x, current_y, font);

                            // Advance X by approx. half the font size per character
                            let advance_mm =
                                Mm(run.text.len() as f64 * font_size_pt * 0.5 * 25.4 / 72.0);
                            current_x += advance_mm;
                        }

                        // Glue (inter-word spacing)
                        LayoutNode::Glue(w) => {
                            let glue_mm = Mm(*w * 25.4 / 72.0);
                            current_x += glue_mm;
                        }
                    }
                }
            }

            // Move down one line
            current_y -= line_height_mm;
        }
    }

    // 5) Serialize PDF to bytes
    let mut buffer = Vec::new();
    {
        let mut writer = BufWriter::new(&mut buffer);
        doc.save(&mut writer).expect("Failed to write PDF");
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;
    use crate::layout::layout;

    #[test]
    fn test_render_pdf_header() {
        let ast = Node::Seq(vec![Node::Text("Hello PDF".into())]);
        let pages = layout(&ast, 100.0, 20.0, 6.0, 6.0);
        let pdf_data = render_pdf(&pages);
        assert!(pdf_data.starts_with(b"%PDF-"));
    }
}
