use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use kariera_domain::{EngagementType, Month, NoteName, Text, ValueError};
use kariera_vault::{
    CreateError, NewEngagement, NewLog, NewNote, NewProject, OrganizationRole, Vault,
};

#[path = "support/vault_copy.rs"]
mod vault_copy;

use vault_copy::copy_of_fixture_vault;

fn name(text: &str) -> Result<NoteName, ValueError> {
    text.parse()
}

fn month(text: &str) -> Result<Month, ValueError> {
    text.parse()
}

fn log(engagement: &str, month_text: &str) -> Result<NewNote, ValueError> {
    Ok(NewNote::Log(NewLog {
        engagement: name(engagement)?,
        month: month(month_text)?,
    }))
}

fn project(engagement: &str, title: &str) -> Result<NewNote, ValueError> {
    Ok(NewNote::Project(NewProject {
        engagement: name(engagement)?,
        title: title.parse::<Text>()?,
    }))
}

fn engagement(
    organization: &str,
    role: OrganizationRole,
    engagement_type: EngagementType,
    start: &str,
) -> Result<NewNote, ValueError> {
    Ok(NewNote::Engagement(NewEngagement {
        organization: name(organization)?,
        role,
        engagement_type,
        start: month(start)?,
    }))
}

/// The vault copy and the result of creating `note` in it.
fn create(
    test: &str,
    note: &NewNote,
) -> Result<(PathBuf, Result<PathBuf, CreateError>), Box<dyn Error>> {
    let root = copy_of_fixture_vault(test)?;
    let created = Vault::load(&root)?.create(note);
    Ok((root, created))
}

fn created_text(test: &str, note: &NewNote, expected_path: &str) -> Result<String, Box<dyn Error>> {
    let (root, created) = create(test, note)?;
    let path = created?;
    if path != root.join(expected_path) {
        return Err(format!("created {}, not {expected_path}", path.display()).into());
    }
    Ok(fs::read_to_string(path)?)
}

fn template(root: &Path, name: &str) -> io::Result<String> {
    fs::read_to_string(root.join("Templates").join(name))
}

fn error_of(test: &str, note: &NewNote) -> Result<CreateError, Box<dyn Error>> {
    match create(test, note)?.1 {
        Err(error) => Ok(error),
        Ok(path) => Err(format!("created {} instead of failing", path.display()).into()),
    }
}

#[test]
fn a_log_is_created_in_the_folder_of_its_year_with_engagement_and_month() {
    let text = created_text(
        "new_log",
        &log("2023-10 Globex", "2026-11").unwrap(),
        "Monthly Logs/2026/2026-11 Globex Log.md",
    )
    .unwrap();

    let expected = template(
        &copy_of_fixture_vault("new_log_template").unwrap(),
        "monthly-log.md",
    )
    .unwrap()
    .replacen("engagement:", "engagement: \"[[2023-10 Globex]]\"", 1)
    .replacen("month:", "month: 2026-11", 1);
    assert_eq!(text, expected);
}

#[test]
fn a_project_is_created_under_its_client_with_engagement_and_english_title() {
    let text = created_text(
        "new_project",
        &project("2023-10 Globex", "Billing").unwrap(),
        "Projects/Globex - Billing.md",
    )
    .unwrap();

    assert!(
        text.starts_with(
            "---\ntype: project\nengagement: \"[[2023-10 Globex]]\"\nend_client:\ntitle:\n  en: Billing\n  ja:\n"
        ),
        "{text}"
    );
}

#[test]
fn a_project_title_that_yaml_would_misread_is_quoted() {
    for (title, written) in [("Yes", "\"Yes\""), ("2026 Relaunch", "\"2026 Relaunch\"")] {
        let text = created_text(
            "new_project_quoted",
            &project("2023-10 Globex", title).unwrap(),
            &format!("Projects/Globex - {title}.md"),
        )
        .unwrap();

        assert!(text.contains(&format!("  en: {written}\n")), "{text}");
    }
}

#[test]
fn a_freelance_engagement_is_created_with_client_type_and_start() {
    let note = engagement(
        "Acme",
        OrganizationRole::Client,
        EngagementType::Freelance,
        "2026-01",
    )
    .unwrap();

    let text = created_text(
        "new_freelance_engagement",
        &note,
        "Engagements/2026-01 Acme.md",
    )
    .unwrap();

    assert!(
        text.starts_with(
            "---\ntype: engagement\nclient: \"[[Acme]]\"\nemployer:\nengagement_type: freelance\nrole:\n"
        ),
        "{text}"
    );
    assert!(text.contains("\nstart: 2026-01\nend:\n"), "{text}");
}

#[test]
fn an_employment_engagement_fills_the_employer() {
    let note = engagement(
        "Müller Logistik",
        OrganizationRole::Employer,
        EngagementType::Employment,
        "2026-04",
    )
    .unwrap();

    let text = created_text(
        "new_employment_engagement",
        &note,
        "Engagements/2026-04 Müller Logistik.md",
    )
    .unwrap();

    assert!(
        text.contains(
            "\nclient:\nemployer: \"[[Müller Logistik]]\"\nengagement_type: employment\n"
        ),
        "{text}"
    );
}

#[test]
fn an_own_business_engagement_takes_the_my_business_organization() {
    let note = engagement(
        "Example Works",
        OrganizationRole::Client,
        EngagementType::OwnBusiness,
        "2026-02",
    )
    .unwrap();

    let text = created_text(
        "new_own_business_engagement",
        &note,
        "Engagements/2026-02 Example Works.md",
    )
    .unwrap();

    assert!(text.contains("client: \"[[Example Works]]\""), "{text}");
}

#[test]
fn a_note_whose_name_is_taken_is_refused_and_the_existing_note_is_kept() {
    let root = copy_of_fixture_vault("refuse_overwrite").unwrap();
    let existing = root.join("Monthly Logs/2025/2025-06 Globex Log.md");
    let before = fs::read_to_string(&existing).unwrap();

    let created = Vault::load(&root)
        .unwrap()
        .create(&log("2023-10 Globex", "2025-06").unwrap());

    assert_eq!(
        created.unwrap_err().to_string(),
        "Monthly Logs/2025/2025-06 Globex Log.md already has that name, ignoring case"
    );
    assert_eq!(fs::read_to_string(existing).unwrap(), before);
}

#[test]
fn a_name_is_taken_even_when_only_the_case_differs() {
    let error = error_of("refuse_case", &project("2023-10 Globex", "search").unwrap()).unwrap();

    assert!(
        matches!(error, CreateError::NameTaken { ref path } if path == Path::new("Projects/Globex - Search.md")),
        "{error}"
    );
}

#[test]
fn an_unknown_engagement_is_refused() {
    let error = error_of(
        "unknown_engagement",
        &log("2030-01 Nowhere", "2030-02").unwrap(),
    )
    .unwrap();

    assert_eq!(
        error.to_string(),
        "no Engagement is named [[2030-01 Nowhere]]"
    );
}

#[test]
fn a_log_outside_the_period_of_its_engagement_is_refused() {
    let before = error_of("month_before", &log("2023-10 Globex", "2023-09").unwrap()).unwrap();
    let after = error_of("month_after", &log("2021-04 Acme", "2023-10").unwrap()).unwrap();

    assert_eq!(
        before.to_string(),
        "[[2023-10 Globex]] runs from 2023-10 to now, so it has no log for 2023-09"
    );
    assert_eq!(
        after.to_string(),
        "[[2021-04 Acme]] runs from 2021-04 to 2023-09, so it has no log for 2023-10"
    );
}

#[test]
fn the_first_and_last_month_of_an_engagement_are_allowed() {
    for month_text in ["2021-04", "2023-09"] {
        let (_, created) =
            create("period_edges", &log("2021-04 Acme", month_text).unwrap()).unwrap();

        assert!(created.is_ok(), "{month_text}: {created:?}");
    }
}

#[test]
fn an_engagement_needs_the_organization_field_its_type_asks_for() {
    let note = engagement(
        "Acme",
        OrganizationRole::Employer,
        EngagementType::Freelance,
        "2026-01",
    )
    .unwrap();

    let error = error_of("role_mismatch", &note).unwrap();

    assert_eq!(
        error.to_string(),
        "an Engagement of type freelance has a client and no employer"
    );
}

#[test]
fn an_engagement_is_refused_for_an_unknown_organization() {
    let note = engagement(
        "Nowhere Inc",
        OrganizationRole::Client,
        EngagementType::Freelance,
        "2026-01",
    )
    .unwrap();

    let error = error_of("unknown_organization", &note).unwrap();

    assert_eq!(
        error.to_string(),
        "no Organization is named [[Nowhere Inc]]"
    );
}

#[test]
fn an_engagement_is_refused_for_an_organization_of_the_wrong_kind() {
    let cases = [
        (
            "Fabrikam",
            OrganizationRole::Client,
            EngagementType::Freelance,
            "client",
        ),
        (
            "Globex",
            OrganizationRole::Client,
            EngagementType::OwnBusiness,
            "my-business",
        ),
        (
            "Globex",
            OrganizationRole::Employer,
            EngagementType::Employment,
            "employer",
        ),
    ];
    for (organization, role, engagement_type, needed) in cases {
        let note = engagement(organization, role, engagement_type, "2026-01").unwrap();

        let error = error_of("wrong_kind", &note).unwrap();

        assert_eq!(
            error.to_string(),
            format!(
                "[[{organization}]] is not of kind {needed}, which its role in the Engagement needs"
            )
        );
    }
}

#[test]
fn a_title_that_is_no_valid_note_name_is_refused() {
    let error = error_of(
        "invalid_name",
        &project("2023-10 Globex", "Billing/Invoices").unwrap(),
    )
    .unwrap();

    assert_eq!(
        error.to_string(),
        "\"Globex - Billing/Invoices\" is not a valid note name: must not contain '/'"
    );
}

#[test]
fn a_missing_template_is_reported_with_its_path() {
    let root = copy_of_fixture_vault("missing_template").unwrap();
    let template = root.join("Templates/monthly-log.md");
    fs::remove_file(&template).unwrap();

    let created = Vault::load(&root)
        .unwrap()
        .create(&log("2023-10 Globex", "2026-11").unwrap());

    assert!(
        created
            .unwrap_err()
            .to_string()
            .starts_with(&format!("cannot read {}", template.display())),
    );
}

#[test]
fn a_template_without_a_field_to_fill_is_reported_and_nothing_is_written() {
    let root = copy_of_fixture_vault("drifted_template").unwrap();
    let path = root.join("Templates/monthly-log.md");
    fs::write(
        &path,
        fs::read_to_string(&path).unwrap().replace("month:\n", ""),
    )
    .unwrap();

    let created = Vault::load(&root)
        .unwrap()
        .create(&log("2023-10 Globex", "2026-11").unwrap());

    assert_eq!(
        created.unwrap_err().to_string(),
        "Templates/monthly-log.md has no empty `month` field to fill"
    );
    assert!(!root.join("Monthly Logs/2026").exists());
}
