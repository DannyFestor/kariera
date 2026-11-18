use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Dict, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

// SIL Open Font License 1.1, see `fonts/`. Inter sets Latin text; BIZ UDPGothic is the one Japanese family of every
// document (owner's decision on issue #7), and the CV falls back to it for Japanese glyphs.
const FONT_FILES: [&[u8]; 5] = [
    include_bytes!("../fonts/Inter-Regular.otf"),
    include_bytes!("../fonts/Inter-SemiBold.otf"),
    include_bytes!("../fonts/Inter-Bold.otf"),
    include_bytes!("../fonts/BIZUDPGothic-Regular.ttf"),
    include_bytes!("../fonts/BIZUDPGothic-Bold.ttf"),
];

/// Everything Typst may see: one template, the inputs and the embedded fonts. No file system, no system fonts, no
/// packages and no clock, so the same document always gives the same PDF.
#[derive(Debug)]
pub(crate) struct EmbeddedWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl EmbeddedWorld {
    pub(crate) fn new(template: &str, inputs: Dict) -> Self {
        let fonts: Vec<Font> = FONT_FILES
            .iter()
            .flat_map(|file| Font::iter(Bytes::new(*file)))
            .collect();
        Self {
            library: LazyHash::new(Library::builder().with_inputs(inputs).build()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            main: Source::new(main_file(), template.to_owned()),
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "`main.typ` is a constant relative path without `..`, which VirtualPath::new always accepts"
)]
fn main_file() -> FileId {
    let path = VirtualPath::new("main.typ").expect("main.typ is a valid virtual path");
    FileId::new(RootedPath::new(VirtualRoot::Project, path))
}

impl World for EmbeddedWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn file(&self, _: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}
