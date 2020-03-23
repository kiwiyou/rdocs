use super::{Mark, parse_marks_forward, Text, parse_text};

#[derive(Debug)]
pub struct ItemSummary {
    name: String,
    attribute: Mark,
    summary: Text,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemKind {
    Module,
    Struct,
    Enum,
    Constant,
    Function,
    Trait,
    Macro,
    Attribute,
    Type,
    Primitive,
    Keyword,
}

#[derive(Debug)]
pub struct SummarySection {
    item_type: ItemKind,
    content: Vec<ItemSummary>,
}

impl ItemSummary {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attribute(&self) -> &Mark {
        &self.attribute
    }

    pub fn summary(&self) -> &Text {
        &self.summary
    }
}

impl SummarySection {
    pub fn item_type(&self) -> ItemKind {
        self.item_type
    }

    pub fn content(&self) -> &[ItemSummary] {
        &self.content
    }
}

use kuchiki::NodeRef;
use html5ever::local_name;
pub(crate) fn parse_summary_forward(sub_item_header: NodeRef) -> Option<(Option<NodeRef>, SummarySection)> {
    let element = &sub_item_header.as_element()?;
    let item_type = match element.attributes.borrow().get("id").unwrap() {
        "modules" => ItemKind::Module,
        "structs" => ItemKind::Struct,
        "enums" => ItemKind::Enum,
        "constants" => ItemKind::Constant,
        "traits" => ItemKind::Trait,
        "functions" => ItemKind::Function,
        "macros" => ItemKind::Macro,
        "attributes" => ItemKind::Attribute,
        "primitives" => ItemKind::Primitive,
        "keywords" => ItemKind::Keyword,
        "types" => ItemKind::Type,
        _ => return None
    };
    let table = sub_item_header.following_siblings().find(|node| {
        node.as_element()
            .map(|element| element.name.local == local_name!("table"))
            .unwrap_or_default()
    })?;
    let section_content = table
        .select("tr")
        .unwrap()
        .flat_map(|tr| {
            let node = tr.as_node();
            let name = node.first_child()?.text_contents();
            let short_docblock = node.last_child()?;
            let (attribute, summary) = parse_short_docblock(short_docblock)?;
            Some(ItemSummary {
                name,
                attribute,
                summary,
            })
        })
        .collect();
    let section = SummarySection {
        item_type,
        content: section_content,
    };
    Some((table.next_sibling(), section))
}

fn parse_short_docblock(short_docblock: NodeRef) -> Option<(Mark, Text)> {
    let (next, mark) = parse_marks_forward(short_docblock);
    
    let p = next?;
    let text = parse_text(&p);
    Some((mark, text))
}