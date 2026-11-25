//! `personal.yaml` as the owner writes it, with the fictional details of `fixtures/personal/`. The fixture is not named
//! `personal.yaml`, which .gitignore keeps out of the repository everywhere.

use std::path::Path;

use kariera_domain::Personal;
use kariera_vault::load_personal;

fn fixture(file: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/personal")
        .join(file)
}

#[test]
fn the_fixture_file_reads_with_its_photo() {
    let personal = load_personal(&fixture("fictional-personal.yaml")).unwrap();

    assert_eq!(
        personal.email.map(|email| email.to_string()).as_deref(),
        Some("robin@example.com")
    );
    assert_eq!(
        personal
            .public
            .location
            .map(|location| location.to_string())
            .as_deref(),
        Some("Japan")
    );
    assert_eq!(
        personal.photo.map(|photo| photo.0),
        Some(std::fs::read(fixture("photo.png")).unwrap())
    );
}

#[test]
fn a_missing_file_holds_no_details() {
    assert_eq!(
        load_personal(&fixture("absent.yaml")).unwrap(),
        Personal::default()
    );
}
