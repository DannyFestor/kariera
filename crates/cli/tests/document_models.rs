//! The document models of the fixture vault. They live here because only the cli may load a vault and build
//! documents from it (ADR 0001).

use std::path::Path;

use kariera_documents::{Edition, Inputs, build};
use kariera_domain::Date;
use kariera_vault::{Vault, load_personal};

#[test]
fn every_document_of_the_fixture_vault() {
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
        let built = build(edition, &inputs).expect("the fixture vault has a Profile");

        insta::assert_debug_snapshot!(
            format!("{}-{}", edition.kind.slug(), edition.language.code()),
            built
        );
    }
}
