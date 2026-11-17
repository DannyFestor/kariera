/// The content of a Paragraphs section (schema §1.6): Markdown paragraphs separated by blank lines, without the
/// blank lines around them. Empty when the section is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Paragraphs(pub String);

/// The content of a Bullets section (schema §1.6), one entry per `- ` line.
pub type Bullets = Vec<crate::Text>;
