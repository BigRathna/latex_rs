// src/lib.rs

// Module declarations
mod ast;
mod backend;
mod expand;
mod layout;
mod lexer;
mod parser;

pub use ast::Node;

/// Compile a TeX-like input string into a PDF byte vector.
///
/// # Parameters
/// - `input`: the TeX source text
///
/// # Returns
/// - `Ok(Vec<u8>)`: PDF bytes on success
/// - `Err(String)`: error message on failure
pub fn compile(input: &str) -> Result<Vec<u8>, String> {
    // 1. Parse input into AST
    let ast = parser::parse(input)?;

    // 2. Expand macros (flatten sequences)
    let expanded = expand::expand_macros(&ast);

    // 2) Compute usable width in points from A4 − 10 mm margins
    let page_w_mm = 210.0;
    let margin_mm = 10.0; // your PDF backend also uses 10 mm margins
    let usable_mm = page_w_mm - 2.0 * margin_mm; // 190 mm
    let pts_per_mm = 72.0 / 25.4; // 1 in = 25.4 mm = 72 pt
    let line_width_pt = usable_mm * pts_per_mm; // ≈ 539 pt

    // 3) Pick your font size and line height in pt
    let font_size_pt = 12.0;
    let line_height_pt = font_size_pt * 1.2; // 14.4 pt leading

    eprintln!(
        "→ Page: {} mm, margins: {} mm → usable: {} mm → {} pt (≈{:.1} mm)",
        page_w_mm,
        margin_mm,
        usable_mm,
        line_width_pt,
        line_width_pt * 25.4 / 72.0,
    );

    let font_size_pt = 12.0;
    let line_height_pt = font_size_pt * 1.2;

    // Instead of hard‐coding CHAR_WIDTH and SPACE_WIDTH in layout.rs,
    // compute them here, in pt:
    let char_width_pt = font_size_pt * 0.5; // a decent avg glyph advance
    let space_width_pt = char_width_pt; // one “space” = one char width

    // 4) Layout in “pt” units
    let pages = layout::layout(
        &expanded,
        line_width_pt,
        line_height_pt,
        char_width_pt,
        space_width_pt,
    );
    // 4. Render to PDF
    let pdf_data = backend::render_pdf(&pages);

    Ok(pdf_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_hello() {
        let tex = "Hello world";
        let pdf = compile(tex).expect("Compile failed");
        assert!(pdf.starts_with(b"%PDF-"), "Output is not a PDF");
    }
}
