//! A link is a HYPERLINK field: docx-rs would number a link relationship from a counter shared by the whole process,
//! so the same document would come out different when it is not the first one rendered.

use docx_rs::Run;

use crate::blocks::text;
use crate::field::field;

pub(crate) fn hyperlink(target: &str, shown: &str, color: &str) -> Vec<Run> {
    let instruction = format!("HYPERLINK \"{}\"", target.replace('"', "%22"));
    field(&instruction, text(shown).underline("single").color(color))
}
