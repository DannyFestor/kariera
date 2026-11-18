//! The documents of the fixture vault as DOCX (issue #8): real paragraphs and tables an agency can edit, and the same
//! bytes each time. The files are read back as outlines, not compared with stored ones: a docx-rs change would break
//! a byte comparison without a broken document.

use std::path::Path;

use kariera_documents::{Document, DocumentKind, Edition, Inputs, build};
use kariera_domain::{Date, Language, Photo};
use kariera_render_docx::{render, renders};
use kariera_vault::{Vault, load_personal};

#[path = "support/docx_package.rs"]
mod docx_package;

use docx_package::{DocxPackage, TestResult};

fn fixture_document(kind: DocumentKind, language: Language) -> TestResult<Document> {
    let vault = Vault::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault"))?;
    let personal = load_personal(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/personal/fictional-personal.yaml"),
    )?;
    let inputs = Inputs {
        career: vault.career(),
        personal: &personal,
        today: "2026-11-18".parse::<Date>()?,
    };
    Ok(build(Edition { kind, language }, &inputs)?.document)
}

fn docx_of(kind: DocumentKind, language: Language) -> TestResult<DocxPackage> {
    DocxPackage::open(&render(&fixture_document(kind, language)?)?)
}

#[test]
fn a_document_renders_to_the_same_bytes_each_time() {
    let document = fixture_document(DocumentKind::Cv, Language::German).unwrap();

    let first = render(&document).unwrap();
    let second = render(&document).unwrap();

    assert!(first == second);
}

#[test]
fn the_files_carry_no_clock() {
    let mut package = docx_of(DocumentKind::Cv, Language::English).unwrap();

    let dates = package.part("docProps/core.xml").unwrap();

    assert_eq!(dates.matches("1970-01-01T00:00:00Z").count(), 2, "{dates}");
    let times = package.modification_times().unwrap();
    assert!(
        times
            .iter()
            .all(|time| time.ends_with("Some(DateTime::default())")),
        "{times:?}"
    );
}

#[test]
fn every_document_with_a_docx_layout_reads_as_paragraphs_and_tables() {
    for edition in Edition::all()
        .into_iter()
        .filter(|edition| renders(edition.kind))
    {
        let outline = docx_of(edition.kind, edition.language)
            .unwrap()
            .outline()
            .unwrap();

        insta::assert_snapshot!(
            format!("{}-{}", edition.kind.slug(), edition.language.code()),
            outline
        );
    }
}

fn pictures(kind: DocumentKind, language: Language) -> TestResult<usize> {
    let body = docx_of(kind, language)?.part("word/document.xml")?;
    Ok(body.matches("<w:drawing>").count())
}

#[test]
fn only_the_german_cv_holds_a_picture_and_no_text_is_an_image() {
    assert_eq!(pictures(DocumentKind::Cv, Language::German).unwrap(), 1);
    assert_eq!(pictures(DocumentKind::Cv, Language::English).unwrap(), 0);
    assert_eq!(
        pictures(DocumentKind::ShokumuKeirekisho, Language::Japanese).unwrap(),
        0
    );
}

#[test]
fn the_fonts_are_named_and_not_embedded() {
    let mut cv = docx_of(DocumentKind::Cv, Language::English).unwrap();
    let mut shokumu = docx_of(DocumentKind::ShokumuKeirekisho, Language::Japanese).unwrap();

    let cv_styles = cv.part("word/styles.xml").unwrap();
    let shokumu_styles = shokumu.part("word/styles.xml").unwrap();

    assert!(cv_styles.contains(r#"w:ascii="Inter""#), "{cv_styles}");
    assert!(
        cv_styles.contains(r#"w:eastAsia="BIZ UDPGothic""#),
        "{cv_styles}"
    );
    assert!(
        shokumu_styles.contains(r#"w:ascii="BIZ UDPGothic""#),
        "{shokumu_styles}"
    );
    let font_parts = cv
        .part_names()
        .unwrap()
        .into_iter()
        .filter(|name| name.contains("font"))
        .collect::<Vec<_>>();
    assert_eq!(font_parts, ["word/fontTable.xml"]);
}

#[test]
fn headings_lists_links_and_page_numbers_are_word_features_an_agency_can_edit() {
    let mut package = docx_of(DocumentKind::Cv, Language::English).unwrap();

    let body = package.part("word/document.xml").unwrap();

    assert!(body.contains(r#"<w:pStyle w:val="Heading1""#));
    assert!(body.contains("<w:numPr>"));
    assert!(body.contains(r#"HYPERLINK "mailto:robin@example.com""#));
    assert!(
        package
            .part("word/footer1.xml")
            .unwrap()
            .contains("NUMPAGES")
    );
}

#[test]
fn the_page_header_of_the_shokumu_keirekisho_skips_the_first_page() {
    let mut package = docx_of(DocumentKind::ShokumuKeirekisho, Language::Japanese).unwrap();

    let body = package.part("word/document.xml").unwrap();

    assert!(body.contains("<w:titlePg"), "{body}");
    assert!(
        package
            .part("word/header1.xml")
            .unwrap()
            .contains("2026年11月18日現在")
    );
}

#[test]
fn only_the_documents_with_a_docx_layout_render() {
    for edition in Edition::all() {
        let rendered = render(&fixture_document(edition.kind, edition.language).unwrap());

        assert_eq!(rendered.is_ok(), renders(edition.kind), "{edition:?}");
    }
}

#[test]
fn a_photo_in_a_format_docx_cannot_hold_is_an_error_not_a_panic() {
    let Document::Cv(mut cv) = fixture_document(DocumentKind::Cv, Language::German).unwrap() else {
        panic!("the German CV is a CV");
    };
    cv.photo = Some(Photo(
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>".to_vec(),
    ));

    let error = render(&Document::Cv(cv))
        .err()
        .ok_or("the photo was read")
        .unwrap();

    assert!(
        error.to_string().starts_with("cannot read the photo"),
        "{error}"
    );
}
