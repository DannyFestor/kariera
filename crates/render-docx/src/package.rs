//! The DOCX package around a layout: page, defaults, and the bytes of the file.

use std::io::Cursor;

use docx_rs::{Docx, PageMargin};

use crate::RenderError;
use crate::theme::{PAGE_HEIGHT, PAGE_WIDTH, SIDE_MARGIN, Typeface};

const EPOCH: &str = "1970-01-01T00:00:00Z";

/// Vertical margins differ per document; the side margins are the same.
pub(crate) struct Page {
    pub(crate) top: i32,
    pub(crate) bottom: i32,
}

pub(crate) fn base(typeface: Typeface, size: usize, page: &Page) -> Docx {
    Docx::new()
        .page_size(PAGE_WIDTH, PAGE_HEIGHT)
        .page_margin(
            PageMargin::new()
                .top(page.top)
                .bottom(page.bottom)
                .left(SIDE_MARGIN)
                .right(SIDE_MARGIN)
                .header(page.top / 2)
                .footer(page.bottom / 2),
        )
        .default_fonts(typeface.run_fonts())
        .default_size(size)
}

/// What the file says about itself, which Word shows under File > Info and uses to pick a spell-checker.
pub(crate) struct Properties<'text> {
    pub(crate) title: &'text str,
    pub(crate) author: &'text str,
    /// A language code such as `de`.
    pub(crate) language: &'text str,
}

/// The dates stay 1970-01-01, as `Docx::new` gives them, and the ZIP entries keep their default date: nothing here
/// adds a clock. docx-rs has no setter for the author or the language, so the parts it writes are replaced after
/// `build`.
pub(crate) fn bytes(docx: Docx, properties: &Properties<'_>) -> Result<Vec<u8>, RenderError> {
    let mut parts = docx.build();
    parts.doc_props.core = core_properties(properties).into_bytes();
    parts.styles = with_default_language(parts.styles, properties.language)?;
    let mut buffer = Cursor::new(Vec::new());
    parts
        .pack(&mut buffer)
        .map_err(|error| RenderError::Pack(error.to_string()))?;
    Ok(buffer.into_inner())
}

fn core_properties(properties: &Properties<'_>) -> String {
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" "#,
            r#"xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" "#,
            r#"xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
            r#"<dcterms:created xsi:type="dcterms:W3CDTF">{epoch}</dcterms:created>"#,
            "<dc:title>{title}</dc:title><dc:creator>{author}</dc:creator>",
            "<cp:lastModifiedBy>{author}</cp:lastModifiedBy><dc:language>{language}</dc:language>",
            r#"<dcterms:modified xsi:type="dcterms:W3CDTF">{epoch}</dcterms:modified>"#,
            "<cp:revision>1</cp:revision></cp:coreProperties>"
        ),
        epoch = EPOCH,
        title = escaped(properties.title),
        author = escaped(properties.author),
        language = escaped(properties.language),
    )
}

/// The end of the run properties every paragraph starts from, in the styles docx-rs writes.
const DEFAULTS_END: &str = "</w:rPr></w:rPrDefault>";

/// The language of the text, and Japanese for the East Asian text, whatever the document's own language.
fn with_default_language(styles: Vec<u8>, language: &str) -> Result<Vec<u8>, RenderError> {
    let styles = String::from_utf8(styles).map_err(|error| RenderError::Pack(error.to_string()))?;
    let lang = format!(
        r#"<w:lang w:val="{}" w:eastAsia="ja" />"#,
        escaped(language)
    );
    if !styles.contains(DEFAULTS_END) {
        return Err(RenderError::Pack(
            "docx-rs wrote its default styles in a form this renderer does not know".to_owned(),
        ));
    }
    Ok(styles
        .replacen(DEFAULTS_END, &format!("{lang}{DEFAULTS_END}"), 1)
        .into_bytes())
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
