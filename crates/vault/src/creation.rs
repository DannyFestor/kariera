//! Creates a note from the vault's `Templates/` folder at the path its type prescribes (schema §1.1), with the
//! fields the arguments determine filled in. Everything else stays as the template has it, for the author to fill.

mod error;
mod fill;
mod plan;
mod request;

use std::fs;
use std::path::{Path, PathBuf};

pub use error::CreateError;
pub use request::{NewEngagement, NewLog, NewNote, NewProject, OrganizationRole};

use kariera_domain::FixedList;

use crate::Vault;
use crate::creation::fill::fill_template;
use crate::creation::plan::Plan;
use crate::write::create_file;

const TEMPLATES_FOLDER: &str = "Templates";

impl Vault {
    /// Writes the note and returns its path. Refuses a name that another note already has, ignoring case
    /// (schema §1.1), so no note is ever overwritten.
    pub fn create(&self, note: &NewNote) -> Result<PathBuf, CreateError> {
        let plan = note.plan(self.career())?;
        self.refuse_taken_name(&plan)?;
        let template = self.read_template(&plan)?;
        let text = fill_template(&template.text, &plan.fills).map_err(|field| {
            CreateError::TemplateField {
                template: template.path,
                field,
            }
        })?;
        let path = self.root().join(plan.path());
        create_file(&path, &text).map_err(|source| CreateError::Unwritable {
            path: path.clone(),
            source,
        })?;
        Ok(path)
    }

    fn refuse_taken_name(&self, plan: &Plan) -> Result<(), CreateError> {
        let wanted = plan.name.as_str().to_lowercase();
        let taken = self
            .notes()
            .iter()
            .find(|existing| existing.name.as_str().to_lowercase() == wanted);
        taken.map_or(Ok(()), |existing| {
            Err(CreateError::NameTaken {
                path: existing.path.clone(),
            })
        })
    }

    fn read_template(&self, plan: &Plan) -> Result<Template, CreateError> {
        let path = Path::new(TEMPLATES_FOLDER).join(format!("{}.md", plan.note_type.slug()));
        let absolute = self.root().join(&path);
        match fs::read_to_string(&absolute) {
            Ok(text) => Ok(Template { path, text }),
            Err(source) => Err(CreateError::Unreadable {
                path: absolute,
                source,
            }),
        }
    }
}

struct Template {
    /// Relative to the vault root.
    path: PathBuf,
    text: String,
}
