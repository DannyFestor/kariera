const BULLET: char = '・';

pub(crate) fn bullet(item: &str) -> String {
    format!("{BULLET}{item}")
}

pub(crate) fn bulleted(items: &[String]) -> String {
    items
        .iter()
        .map(|item| bullet(item))
        .collect::<Vec<_>>()
        .join("\n")
}
