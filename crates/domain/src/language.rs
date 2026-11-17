#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Language {
    English,
    Japanese,
    German,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::English, Self::Japanese, Self::German];

    /// The key of the language in a translatable map (schema §1.5).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Japanese => "ja",
            Self::German => "de",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn codes_are_the_keys_of_a_translatable_map() {
        let codes = Language::ALL.map(Language::code);

        assert_eq!(codes, ["en", "ja", "de"]);
    }
}
