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

pub struct Text {
    components: Vec<TextComponent>,
}

impl Text {
    pub fn components(&self) -> &[TextComponent] {
        &self.components
    }
}

#[derive(Clone, Copy)]
pub struct SectionRef(usize);

pub struct Section {
    title: Option<Text>,
    content: SectionContent,
    parent: Option<SectionRef>,
    children: Vec<SectionRef>,
}

impl Section {
    pub fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    pub fn content(&self) -> &SectionContent {
        &self.content
    }

    pub fn parent_ref(&self) -> Option<SectionRef> {
        self.parent
    }

    pub fn children_ref(&self) -> &[SectionRef] {
        &self.children
    }
}

pub enum SectionContent {
    Details(Vec<Paragraph>),
    Modules(Vec<ModuleItem>),
    Reexports(Vec<ExportItem>),
}

pub enum Paragraph {
    Text(Text),
    List(Vec<Text>),
    Code(String),
    InvalidCode(String),
}

pub struct ExportItem {
    use_definition: String,
}

impl ExportItem {
    pub fn definition(&self) -> &str {
        &self.use_definition
    }
}

pub struct ModuleItem {
    name: String,
    attribute: Attribute,
    summary: Text,
}

impl ModuleItem {
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

pub struct Attribute {
    stability: String,
    features: String,
}

impl Attribute {
    pub fn stability(&self) -> &str {
        &self.stability
    }

    pub fn features(&self) -> &str {
        &self.features
    }
}

pub struct Document {
    title: String,
    sections: Vec<Section>,
}

impl Document {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn section_of(&self, reference: SectionRef) -> Option<&Section> {
        self.sections.get(reference.0)
    }
}