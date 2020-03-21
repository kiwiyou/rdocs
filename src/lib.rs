use html5ever::local_name;
use html5ever::tendril::TendrilSink;
use kuchiki::{iter::NodeEdge, NodeData, NodeRef};

#[derive(Debug, Clone)]
pub struct TextComponent {
    text: String,
    code: bool,
    italic: bool,
    bold: bool,
    link: Option<String>,
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

#[derive(Debug)]
pub struct Text {
    components: Vec<TextComponent>,
}

impl Text {
    pub fn components(&self) -> &[TextComponent] {
        &self.components
    }
}

#[derive(Debug)]
pub struct Section {
    title: Option<Text>,
    content: Vec<Paragraph>,
}

impl Section {
    pub fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    pub fn content(&self) -> &[Paragraph] {
        &self.content
    }
}

#[derive(Debug)]
pub enum Paragraph {
    Text(Text),
    List(Vec<Text>),
    Code(String),
    InvalidCode(String),
    SubSection(Box<Section>),
}

#[derive(Debug)]
pub struct ExportItem(String);

impl ExportItem {
    pub fn definition(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct SubItem {
    name: String,
    attribute: Attribute,
    summary: Text,
}

impl SubItem {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attribute(&self) -> &Attribute {
        &self.attribute
    }

    pub fn summary(&self) -> &Text {
        &self.summary
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SubItemType {
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
pub struct SubItemSection {
    item_type: SubItemType,
    content: Vec<SubItem>,
}

impl SubItemSection {
    pub fn item_type(&self) -> SubItemType {
        self.item_type
    }

    pub fn content(&self) -> &[SubItem] {
        &self.content
    }
}

#[derive(Debug)]
pub struct Attribute {
    stability: String,
    features: String,
    deprecated: String,
}

impl Attribute {
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

#[derive(Debug)]
pub struct Document {
    title: String,
    declaration: Option<String>,
    description: Vec<Section>,
    exports: Vec<ExportItem>,
    sub_items: Vec<SubItemSection>,
}

impl Document {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn declaration(&self) -> Option<&str> {
        self.declaration.as_deref()
    }

    pub fn description(&self) -> &[Section] {
        &self.description
    }

    pub fn exports(&self) -> &[ExportItem] {
        &self.exports
    }

    pub fn sub_items(&self) -> &[SubItemSection] {
        &self.sub_items
    }
}

#[derive(Default)]
pub struct DocsClient {
    client: reqwest::Client,
}

impl DocsClient {
    pub async fn get_document(&self, package_name: &str, path: &str) -> Option<Document> {
        let url = self.get_path_url(package_name, path).await?;
        let data = reqwest::get(&url).await.ok()?.text().await.ok()?;
        parse_rust_document(&data)
    }

    async fn get_path_url(&self, package_name: &str, path: &str) -> Option<String> {
        let path_parts: Vec<&str> = path.splitn(2, "::").collect();
        let url = get_docs_rs_url(package_name, path_parts[0]);
        if path_parts.len() == 2 {
            self.find_module(&url, path_parts[1])
                .await
                .or(self.find_sub_item(&url, path_parts[1]).await)
        } else {
            Some(url + "/index.html")
        }
    }

    async fn find_sub_item(&self, url: &str, sub_path: &str) -> Option<String> {
        let index_url = url.to_owned() + "/all.html";
        let data = self
            .client
            .get(&index_url)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;
        let index_page = kuchiki::parse_html().one(data.as_ref());
        let link = index_page
            .select(".docblock > li > a")
            .unwrap()
            .find(|a| a.text_contents() == sub_path)?;
        let attributes = link.as_node().as_element()?.attributes.borrow();
        Some(url.to_owned() + "/" + attributes.get("href")?)
    }

    async fn find_module(&self, url: &str, sub_path: &str) -> Option<String> {
        let mut check_url = url.to_owned();
        for module in sub_path.split("::") {
            check_url.push('/');
            check_url.push_str(module);
        }
        let data = self.client.head(&check_url).send().await.ok()?;
        if data.status().is_success() {
            Some(check_url)
        } else {
            None
        }
    }
}

fn get_docs_rs_url(package_name: &str, crate_name: &str) -> String {
    format!(
        "https://docs.rs/{package}/*/{crate}",
        package = package_name,
        crate = crate_name
    )
}

fn parse_rust_document(input: &str) -> Option<Document> {
    use kuchiki::traits::*;
    let dom = kuchiki::parse_html().one(input);
    let main = dom.select_first("#main").ok()?;

    let fqn = main.as_node().select_first(".fqn").ok()?;
    let title = fqn.as_node().last_child()?.text_contents();

    let type_decl = fqn
        .as_node()
        .following_siblings()
        .select(".type-decl")
        .unwrap()
        .next();
    let mut foremost = type_decl
        .as_ref()
        .map(|n| n.as_node())
        .unwrap_or(fqn.as_node());

    let docblock = foremost
        .following_siblings()
        .select(".docblock")
        .unwrap()
        .next();
    let description = docblock
        .as_ref()
        .map(|n| parse_docblock(n.as_node()))
        .flatten()
        .unwrap_or_default();
    foremost = docblock.as_ref().map(|n| n.as_node()).unwrap_or(foremost);

    let mut current_tag = foremost.next_sibling();
    let mut re_exports = Vec::new();
    let mut sub_item_sections = Vec::new();
    while let Some(current) = current_tag {
        let element = match current.as_element() {
            Some(element) => element,
            None => {
                current_tag = current.next_sibling();
                continue;
            }
        };
        let attributes = element.attributes.borrow();
        match attributes.get("class") {
            Some("section-header") => {
                if attributes.get("id") == Some("reexports") {
                    let (new_current, exports) = parse_exports(&current)?;
                    re_exports = exports;
                    current_tag = new_current.next_sibling();
                } else {
                    let (new_current, sub_item_section) = parse_sub_items(&current)?;
                    sub_item_sections.push(sub_item_section);
                    current_tag = new_current.next_sibling();
                }
            }
            _ => current_tag = current.next_sibling(),
        }
    }

    Some(Document {
        title,
        declaration: None,
        description,
        exports: re_exports,
        sub_items: sub_item_sections,
    })
}

fn parse_exports(export_header: &NodeRef) -> Option<(NodeRef, Vec<ExportItem>)> {
    let table = export_header.next_sibling()?;
    let exports = table
        .select("tr > td > code")
        .unwrap()
        .map(|element| ExportItem(element.text_contents()))
        .collect();
    Some((table, exports))
}

fn parse_sub_items(sub_item_header: &NodeRef) -> Option<(NodeRef, SubItemSection)> {
    let element = &sub_item_header.as_element()?;
    let item_type = match element.attributes.borrow().get("id").unwrap() {
        "primitives" => SubItemType::Primitive,
        "modules" => SubItemType::Module,
        "structs" => SubItemType::Struct,
        "enums" => SubItemType::Enum,
        "constants" => SubItemType::Constant,
        "traits" => SubItemType::Trait,
        "functions" => SubItemType::Function,
        "macros" => SubItemType::Macro,
        "attributes" => SubItemType::Attribute,
        "keywords" => SubItemType::Keyword,
        "types" => SubItemType::Type,
        name @ _ => panic!("unknown sub item type found: {}", name),
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
            let (attribute, summary) = parse_short_docblock(&short_docblock)?;
            Some(SubItem {
                name,
                attribute,
                summary,
            })
        })
        .collect();
    let section = SubItemSection {
        item_type,
        content: section_content,
    };
    Some((table, section))
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

fn parse_docblock(docblock: &NodeRef) -> Option<Vec<Section>> {
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

fn parse_short_docblock(short_docblock: &NodeRef) -> Option<(Attribute, Text)> {
    let attribute = Attribute {
        stability: short_docblock
            .select_first(".unstable")
            .map(|span| span.text_contents())
            .unwrap_or_default(),
        features: short_docblock
            .select_first(".portability")
            .map(|span| span.text_contents())
            .unwrap_or_default(),
        deprecated: short_docblock
            .select_first(".deprecated")
            .map(|span| span.text_contents())
            .unwrap_or_default(),
    };
    let p = short_docblock.select_first("p").ok()?;
    let text = parse_text(p.as_node());
    Some((attribute, text))
}

fn parse_text(node: &NodeRef) -> Text {
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
