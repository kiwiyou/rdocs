mod text;
mod reexport;
mod summary;
mod mark;
mod document;
mod implementation;

pub use text::*;
pub use reexport::*;
pub use summary::*;
pub use mark::*;
pub use document::*;
pub use implementation::*;

pub type Code = String;

#[derive(Debug)]
pub struct SimpleItem {
    declaration: Code,
    mark: Mark,
    description: Vec<Section>,
}

pub trait Described {
    fn description(&self) -> &[Section];
}

pub trait Exportable {
    fn re_exports(&self) -> &[ExportItem];
}

pub trait ItemContainer {
    fn sub_item(&self) -> &[SummarySection];
}

pub trait Declared {
    fn declaration(&self) -> &Code;
}

pub trait Implementable {
    fn trait_impls(&self) -> &[Implementation];
    fn auto_impls(&self) -> &[Implementation];
    fn blanket_impls(&self) -> &[Implementation];
}

pub trait Marked {
    fn mark(&self) -> &Mark;
}

impl Described for SimpleItem {
    fn description(&self) -> &[Section] {
        &self.description
    }
}

impl Marked for SimpleItem {
    fn mark(&self) -> &Mark {
        &self.mark
    }
}

use kuchiki::{NodeRef, NodeData, iter::NodeEdge};
use html5ever::local_name;
pub(crate) fn parse_simple_item_forward(head: NodeRef) -> Option<(Option<NodeRef>, SimpleItem)> {
    let declaration = parse_generic_code(&head);
    let head = head.next_sibling().and_then(skip_toggle_wrapper);

    let (head, mark) = if let Some(head) = head {
        parse_marks_forward(head)
    } else {
        (None, Mark::default())
    };

    let head = head.and_then(skip_toggle_wrapper);
    let (head, description) = if let Some(head) = head {
        (head.next_sibling(), parse_docblock(&head)?)
    } else {
        (None, Vec::new())
    };

    Some((head, SimpleItem {
        declaration,
        mark,
        description
    }))
}

pub(crate) fn parse_generic_code(pre: &NodeRef) -> Code {
    let mut output = Code::new();
    for edge in pre.traverse() {
        if let NodeEdge::Start(node_start) = edge {
            match node_start.data() {
                NodeData::Text(text) => output.push_str(&text.borrow()),
                NodeData::Element(element) => {
                    match element.name.local {
                        local_name!("br") => output.push('\n'),
                        local_name!("span") => {
                            let attributes = element.attributes.borrow();
                            let has_newline_option = attributes.get("class")
                            .map(|class| class.contains("fmt-newline"))
                            .unwrap_or(false);
                            if has_newline_option {
                                output.push('\n');
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    output
}

pub(crate) fn skip_uninformative(head: NodeRef) -> Option<NodeRef> {
    head.inclusive_following_siblings().find(|n| {
        if let Some(element) = n.as_element() {
            if let Some(class) = element.attributes.borrow().get("class") {
                class.contains("toggle")
            } else {
                false
            }
        } else {
            false
        }
    })
}