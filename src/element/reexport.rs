use super::{Declared, Code};

#[derive(Debug)]
pub struct ExportItem(Code);

impl Declared for ExportItem {
    fn declaration(&self) -> &Code {
        &self.0
    }
}

use kuchiki::NodeRef;
pub(crate) fn parse_exports_forward(export_header: NodeRef) -> Option<(Option<NodeRef>, Vec<ExportItem>)> {
    let table = export_header.next_sibling()?;
    let exports = table
        .select("tr > td > code")
        .unwrap()
        .map(|element| ExportItem(element.text_contents()))
        .collect();
    Some((table.next_sibling(), exports))
}