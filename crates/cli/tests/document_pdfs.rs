//! Every document of the fixture vault renders to a PDF, the same bytes each time (issue #7). The PDFs are not
//! compared with stored ones: a Typst or font change would break such a test without a broken document.

use std::error::Error;
use std::path::Path;

use kariera_documents::{Audience, Contact, Cv, Document, Edition, Inputs, build};
use kariera_domain::{Date, Personal};
use kariera_render_typst::{Pdf, render};
use kariera_vault::{Vault, load_personal};

type TestResult<T> = Result<T, Box<dyn Error>>;

fn fixture_vault() -> TestResult<Vault> {
    Ok(Vault::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault"),
    )?)
}

fn fixture_personal() -> TestResult<Personal> {
    Ok(load_personal(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/personal/fictional-personal.yaml"),
    )?)
}

fn built(vault: &Vault, personal: &Personal, edition: Edition) -> TestResult<Document> {
    let inputs = Inputs {
        career: vault.career(),
        personal,
        today: "2026-11-18".parse::<Date>()?,
    };
    Ok(build(edition, &inputs)?.document)
}

fn pdf(document: &Document) -> TestResult<Pdf> {
    Ok(render(document)?)
}

fn public_editions() -> Vec<Edition> {
    Edition::all()
        .into_iter()
        .filter(|edition| edition.audience == Audience::Public)
        .collect()
}

#[test]
fn every_document_of_the_fixture_vault_renders_the_same_pdf_each_time() {
    let vault = fixture_vault().unwrap();
    let personal = fixture_personal().unwrap();

    for edition in Edition::all() {
        let document = built(&vault, &personal, edition).unwrap();

        let first = pdf(&document).unwrap();
        let second = pdf(&document).unwrap();

        assert!(first.bytes.starts_with(b"%PDF-"), "{edition:?}");
        assert!(first.pages >= 1, "{edition:?}");
        assert!(
            first == second,
            "{edition:?} renders differently the second time"
        );
    }
}

/// A key added to `personal.yaml` outside `public:`, once the fixture sets it, cannot change a public PDF without
/// failing this test.
#[test]
fn a_public_pdf_is_the_same_whatever_personal_yaml_holds_outside_its_public_map() {
    let vault = fixture_vault().unwrap();
    let every_key = fixture_personal().unwrap();
    let public_only = Personal {
        public: every_key.public.clone(),
        ..Personal::default()
    };

    for edition in public_editions() {
        let document = built(&vault, &every_key, edition).unwrap();
        let from_public_only = built(&vault, &public_only, edition).unwrap();

        assert_eq!(document, from_public_only, "{edition:?}");
        assert!(
            pdf(&document).unwrap() == pdf(&from_public_only).unwrap(),
            "{edition:?} prints something from outside `public:`"
        );
    }
}

/// The phone, the address, the birth date, the nationality and the photo are left out; the email and the location
/// are the public ones.
#[test]
fn a_public_document_is_the_private_one_with_only_the_public_email_and_location() {
    let vault = fixture_vault().unwrap();
    let personal = fixture_personal().unwrap();

    for edition in public_editions() {
        let private = built(
            &vault,
            &personal,
            Edition {
                audience: Audience::Private,
                ..edition
            },
        )
        .unwrap();

        assert_eq!(
            built(&vault, &personal, edition).unwrap(),
            with_public_details(private),
            "{edition:?}"
        );
    }
}

fn with_public_details(private: Document) -> Document {
    match private {
        Document::Cv(cv) => Document::Cv(Cv {
            contact: Contact {
                email: Some("hello@example.com".to_owned()),
                location: Some("Japan".to_owned()),
                phone: None,
                ..cv.contact
            },
            photo: None,
            personal_details: None,
            ..cv
        }),
        other @ (Document::ShokumuKeirekisho(_)
        | Document::Rirekisho(_)
        | Document::SkillSheet(_)) => other,
    }
}
