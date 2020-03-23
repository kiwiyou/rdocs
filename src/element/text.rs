use super::Code;

#[derive(Debug)]
pub enum Paragraph {
    Text(Text),
    List(Vec<Text>),
    Code(Code),
    InvalidCode(Code),
    SubSection(Box<Section>),
}

#[derive(Debug)]
pub struct Section {
    title: Option<Text>,
    content: Vec<Paragraph>,
}


#[derive(Debug, Clone)]
pub struct TextComponent {
    text: String,
    code: bool,
    italic: bool,
    bold: bool,
    link: Option<String>,
}

#[derive(Debug)]
pub struct Text {
    components: Vec<TextComponent>,
}

impl Section {
    pub fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    pub fn content(&self) -> &[Paragraph] {
        &self.content
    }
}

impl TextComponent {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_code(&self) -> bool {
        self.code
    }

    pub fn is_italic(&self) -> bool {
        self.italic
    }

    pub fn is_bold(&self) -> bool {
        self.bold
    }

    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }
}

impl Text {
    pub fn components(&self) -> &[TextComponent] {
        &self.components
    }
}

use kuchiki::{NodeRef, NodeData, iter::NodeEdge};
use html5ever::local_name;
pub(crate) fn parse_text(node: &NodeRef) -> Text {
    let mut code_stack = 0;
    let mut italic_stack = 0;
    let mut bold_stack = 0;
    let mut link_stack = Vec::new();
    let mut components = Vec::new();
    for edge in node.traverse() {
        match edge {
            NodeEdge::Start(node_start) => match node_start.data() {
                NodeData::Element(element) => match element.name.local {
                    local_name!("code") => code_stack += 1,
                    local_name!("strong") => bold_stack += 1,
                    local_name!("em") => italic_stack += 1,
                    local_name!("a") => {
                        link_stack.push(element.attributes.borrow().get("href").map(str::to_owned))
                    }
                    _ => {}
                },
                NodeData::Text(text) => {
                    components.push(TextComponent {
                        text: text.borrow().clone(),
                        code: code_stack > 0,
                        italic: italic_stack > 0,
                        bold: bold_stack > 0,
                        link: link_stack.last().cloned().flatten(),
                    });
                }
                _ => {}
            },
            NodeEdge::End(node_end) => {
                if let Some(element) = node_end.as_element() {
                    match element.name.local {
                        local_name!("code") => code_stack -= 1,
                        local_name!("strong") => bold_stack -= 1,
                        local_name!("em") => italic_stack -= 1,
                        local_name!("a") => {
                            link_stack.pop();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Text { components }
}

macro_rules! handle_heading_depth {
    ($depth:literal, $stack:ident, $doc_node:ident, $title:ident, $content:ident) => {{
        // heading depth is $depth - 2, therefore save current state
        // and work on a new state
        if $stack.len() < $depth - 1 {
            $stack.push(Section {
                title: $title,
                content: $content,
            });
        } else if $stack.len() >= $depth - 1 {
            let mut previous_stage = $stack.pop().unwrap();
            previous_stage
                .content
                .push(Paragraph::SubSection(Box::new(Section {
                    title: $title,
                    content: $content,
                })));
            while $stack.len() >= $depth - 1 {
                let mut parent_stage = $stack.pop().unwrap();
                parent_stage
                    .content
                    .push(Paragraph::SubSection(Box::new(previous_stage)));
                previous_stage = parent_stage;
            }
            $stack.push(previous_stage);
        }
        $title = Some(parse_text(&$doc_node));
        $content = Vec::new();
    }};
}

pub(crate) fn parse_docblock(docblock: &NodeRef) -> Option<Vec<Section>> {
    let mut stack = Vec::new();
    let mut sections = Vec::new();
    let mut content = Vec::new();
    let mut title = None;
    for doc_node in docblock.children() {
        let element = match doc_node.as_element() {
            Some(element) => element,
            None => continue,
        };
        match element.name.local {
            local_name!("p") => {
                let text = parse_text(&doc_node);
                content.push(Paragraph::Text(text));
            }
            local_name!("ul") | local_name!("ol") => {
                let list = doc_node
                    .children()
                    .map(|child| parse_text(&child))
                    .filter(|text| !text.components.is_empty())
                    .collect();
                content.push(Paragraph::List(list));
            }
            local_name!("pre") => {
                let code = doc_node.text_contents();
                content.push(Paragraph::Code(code));
            }
            local_name!("div") => {
                let pre = doc_node.first_child()?;
                let code = pre.text_contents();
                if pre
                    .as_element()?
                    .attributes
                    .borrow()
                    .get("class")?
                    .contains("ignore")
                {
                    content.push(Paragraph::InvalidCode(code));
                } else {
                    content.push(Paragraph::Code(code));
                }
            }
            local_name!("h1") => {
                if !stack.is_empty() {
                    let mut previous_stage: Section = stack.pop().unwrap();
                    previous_stage
                        .content
                        .push(Paragraph::SubSection(Box::new(Section { title, content })));
                    while !stack.is_empty() {
                        let mut parent_stage = stack.pop().unwrap();
                        parent_stage
                            .content
                            .push(Paragraph::SubSection(Box::new(previous_stage)));
                        previous_stage = parent_stage;
                    }
                    sections.push(previous_stage);
                } else {
                    sections.push(Section { title, content });
                }
                title = Some(parse_text(&doc_node));
                content = Vec::new();
            }
            local_name!("h2") => handle_heading_depth!(2, stack, doc_node, title, content),
            local_name!("h3") => handle_heading_depth!(3, stack, doc_node, title, content),
            local_name!("h4") => handle_heading_depth!(4, stack, doc_node, title, content),
            local_name!("h5") => handle_heading_depth!(5, stack, doc_node, title, content),
            local_name!("h6") => handle_heading_depth!(6, stack, doc_node, title, content),
            _ => {}
        }
    }
    if !stack.is_empty() {
        let mut previous_stage = stack.pop().unwrap();
        previous_stage
            .content
            .push(Paragraph::SubSection(Box::new(Section { title, content })));
        while !stack.is_empty() {
            let mut parent_stage = stack.pop().unwrap();
            parent_stage
                .content
                .push(Paragraph::SubSection(Box::new(previous_stage)));
            previous_stage = parent_stage;
        }
        sections.push(previous_stage);
    } else {
        sections.push(Section { title, content });
    }

    Some(sections)
}