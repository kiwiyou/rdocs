use super::{Code, SimpleItem, parse_generic_code, skip_uninformative, parse_simple_item_forward};

#[derive(Debug)]
pub struct Implementation {
    impl_code: Code,
    methods: Vec<SimpleItem>,
    assoc_types: Vec<SimpleItem>,
}

impl Implementation {
    pub fn impl_code(&self) -> &Code {
        &self.impl_code
    }

    pub fn methods(&self) -> &[SimpleItem] {
        &self.methods
    }

    pub fn associated_types(&self) -> &[SimpleItem] {
        &self.assoc_types
    }
}

use kuchiki::NodeRef;
pub(crate) fn parse_implementation_forward(head: NodeRef) -> Option<(Option<NodeRef>, Implementation)> {
    let impl_code = parse_generic_code(&head.first_child()?);
    let mut assoc_types = Vec::new();
    let mut methods = Vec::new();
    let head = head.next_sibling();
    if let Some(items) = head.as_ref().and_then(|sibling| sibling.first_child()) {
        let mut item_head = skip_uninformative(items);
        while let Some(decl_node) = item_head {
            if decl_node.as_element().map_or(false, |e| e.attributes.borrow().get("class") == Some("class")) {
                let (new_head, item) = parse_simple_item_forward(decl_node)?;
                assoc_types.push(item);
                item_head = new_head.and_then(skip_uninformative);
            } else {
                item_head = Some(decl_node);
                break;
            }
        }
        while let Some(decl_node) = item_head {
            let (new_head, item) = parse_simple_item_forward(decl_node)?;
            methods.push(item);
            item_head = new_head.and_then(skip_uninformative);
        }
    }

    Some((head, Implementation {
        impl_code,
        methods,
        assoc_types
    }))
}