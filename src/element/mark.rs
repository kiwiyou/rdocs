#[derive(Debug, Default)]
pub struct Mark {
    stability: String,
    features: String,
    deprecated: String,
}

impl Mark {
    pub fn stability(&self) -> &str {
        &self.stability
    }

    pub fn features(&self) -> &str {
        &self.features
    }

    pub fn deprecated(&self) -> &str {
        &self.deprecated
    }
}

use kuchiki::NodeRef;
pub(crate) fn parse_marks_forward(node: NodeRef) -> (Option<NodeRef>, Mark) {
    let mut attribute = Mark::default();
    let mut next = Some(node);
    while let Some(node) = next.as_ref() {
        if let Some(element) = node.as_element() {
            match element.attributes.borrow().get("class") {
                Some(".unstable") => {
                    attribute.stability = node.text_contents();
                }
                Some(".portability") => {
                    attribute.features = node.text_contents();
                }
                Some(".deprecated") => {
                    attribute.deprecated = node.text_contents();
                }
                _ => break
            }
            next = node.next_sibling();
        } else {
            break;
        }
    }

    (next, attribute)
}