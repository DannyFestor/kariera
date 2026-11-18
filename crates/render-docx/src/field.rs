//! Word fields written as the runs of a complex field. docx-rs writes link relationships and page fields from counters
//! it shares with the whole process or without a style, so the layouts write the fields themselves.

use docx_rs::{FieldCharType, InstrText, Run};

/// A field with its instruction and the run that shows its result until Word recalculates it.
pub(crate) fn field(instruction: &str, shown: Run) -> Vec<Run> {
    vec![
        Run::new().add_field_char(FieldCharType::Begin, false),
        Run::new().add_instr_text(InstrText::Unsupported(format!(" {instruction} "))),
        Run::new().add_field_char(FieldCharType::Separate, false),
        shown,
        Run::new().add_field_char(FieldCharType::End, false),
    ]
}
