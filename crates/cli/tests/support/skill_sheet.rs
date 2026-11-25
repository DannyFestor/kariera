//! The Skill Sheet of the fixture vault and the ways the XLSX tests look at it.

use std::path::Path;

use kariera_documents::{Document, Edition, Inputs, SkillSheet, build};
use kariera_domain::Date;
use kariera_render_xlsx::render;
use kariera_vault::{Vault, load_personal};

use crate::xlsx::Xlsx;

pub(crate) fn fixture_skill_sheet() -> SkillSheet {
    let vault = Vault::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault"))
        .expect("the fixture vault loads");
    let personal = load_personal(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/personal/fictional-personal.yaml"),
    )
    .expect("the fixture personal.yaml reads");
    let inputs = Inputs {
        career: vault.career(),
        personal: &personal,
        today: "2026-11-18".parse::<Date>().unwrap(),
    };
    let edition = Edition::all()
        .into_iter()
        .find(|edition| edition.kind.slug() == "skill-sheet")
        .expect("the Skill Sheet is an edition");
    match build(edition, &inputs).unwrap().document {
        Document::SkillSheet(sheet) => sheet,
        other @ (Document::Cv(_) | Document::ShokumuKeirekisho(_) | Document::Rirekisho(_)) => {
            panic!("expected the Skill Sheet, got {other:?}")
        }
    }
}

pub(crate) fn rendered(sheet: &SkillSheet) -> Xlsx {
    Xlsx::open(render(sheet).expect("the Skill Sheet renders"))
}

/// The skill table below its header: category, skill, experience and last use of each row.
pub(crate) fn skill_rows(xlsx: &Xlsx) -> Vec<[Option<String>; 4]> {
    let (header, _) = xlsx.find("分類").expect("the skill table has a header");
    let mut rows = Vec::new();
    for row in header + 1.. {
        let columns = [0, 3, 7, 9].map(|column| xlsx.cell(row, column));
        if columns[1].is_none() {
            break;
        }
        rows.push(columns);
    }
    rows
}

pub(crate) fn project_top(xlsx: &Xlsx, number: u32) -> u32 {
    xlsx.find(&format!("No. {number}"))
        .unwrap_or_else(|| panic!("the sheet has a block for Project {number}"))
        .0
}

pub(crate) fn texts_between(xlsx: &Xlsx, first_row: u32, end_row: u32) -> Vec<String> {
    xlsx.cells()
        .into_iter()
        .filter(|(row, ..)| (first_row..end_row).contains(row))
        .map(|(.., text)| text)
        .collect()
}
