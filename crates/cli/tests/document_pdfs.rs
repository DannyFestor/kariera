//! Every document of the fixture vault renders to a PDF, the same bytes each time (issue #7). The PDFs are not
//! compared with stored ones: a Typst or font change would break such a test without a broken document.

use std::path::Path;

use kariera_documents::{Edition, Inputs, build};
use kariera_domain::Date;
use kariera_render_typst::render;
use kariera_vault::{Vault, load_personal};

#[test]
fn every_document_of_the_fixture_vault_renders_the_same_pdf_each_time() {
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

    for edition in Edition::all() {
        let document = build(edition, &inputs).unwrap().document;

        let first = render(&document).unwrap_or_else(|error| panic!("{edition:?}: {error}"));
        let second = render(&document).unwrap();

        assert!(first.bytes.starts_with(b"%PDF-"), "{edition:?}");
        assert!(first.pages >= 1, "{edition:?}");
        assert!(
            first == second,
            "{edition:?} renders differently the second time"
        );
    }
}
