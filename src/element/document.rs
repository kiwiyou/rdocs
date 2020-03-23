use super::*;

#[derive(Debug)]
pub enum DocumentKind {
    Crate(Module),
    Module(Module),
    Struct(Struct),
    Enum(Enum),
    Constant(Code),
    Function(Code),
    Trait(Trait),
    Macro(Code),
    Attribute(Code),
    Type(Code),
    Primitive(Primitive),
    Keyword,
}

#[derive(Debug)]
pub struct Document {
    title: String,
    mark: Mark,
    description: Vec<Section>,
    kind: DocumentKind,
}

#[derive(Debug, Default)]
pub struct Module {
    re_exports: Vec<ExportItem>,
    sub_item: Vec<SummarySection>,
}

#[derive(Debug)]
pub struct Struct {
    declaration: Code,
    fields: Vec<SimpleItem>,
    methods: Vec<Implementation>,
    trait_impl: Vec<Implementation>,
    auto_impl: Vec<Implementation>,
    blanket: Vec<Implementation>,
}

#[derive(Debug)]
pub struct Trait {
    declaration: Code,
    assoc_types: Vec<SimpleItem>,
    required: Vec<SimpleItem>,
    foreigns: Vec<Implementation>,
    implementors: Vec<Implementation>,
}

#[derive(Debug)]
pub struct Enum {
    declaration: Code,
    variants: Vec<SimpleItem>,
    trait_impl: Vec<Implementation>,
    auto_impl: Vec<Implementation>,
    blanket: Vec<Implementation>,
}

#[derive(Debug)]
pub struct Primitive {
    methods: Vec<Implementation>,
    trait_impl: Vec<Implementation>,
    auto_impl: Vec<Implementation>,
    blanket: Vec<Implementation>,
}

impl Document {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn kind(&self) -> &DocumentKind {
        &self.kind
    }
}

impl Described for Document {
    fn description(&self) -> &[Section] {
        &self.description
    }
}

impl Marked for Document {
    fn mark(&self) -> &Mark {
        &self.mark
    }
}

impl Exportable for Module {
    fn re_exports(&self) -> &[ExportItem] {
        &self.re_exports
    }
}

impl ItemContainer for Module {
    fn sub_item(&self) -> &[SummarySection] {
        &self.sub_item
    }
}

impl Struct {
    pub fn methods(&self) -> &[Implementation] {
        &self.methods
    }

    pub fn fields(&self) -> &[SimpleItem] {
        &self.fields
    }
}

impl Implementable for Struct {
    fn trait_impls(&self) -> &[Implementation] {
        &self.trait_impl
    }

    fn auto_impls(&self) -> &[Implementation] {
        &self.auto_impl
    }

    fn blanket_impls(&self) -> &[Implementation] {
        &self.blanket
    }
}

impl Declared for Struct {
    fn declaration(&self) -> &Code {
        &self.declaration
    }
}

impl Trait {
    pub fn associated_types(&self) -> &[SimpleItem] {
        &self.assoc_types
    }

    pub fn required_methods(&self) -> &[SimpleItem] {
        &self.required
    }

    pub fn foreign_implementations(&self) -> &[Implementation] {
        &self.foreigns
    }

    pub fn implementors(&self) -> &[Implementation] {
        &self.implementors
    }
}

impl Declared for Trait {
    fn declaration(&self) -> &Code {
        &self.declaration
    }
}

impl Enum {
    pub fn variants(&self) -> &[SimpleItem] {
        &self.variants
    }
}

impl Declared for Enum {
    fn declaration(&self) -> &Code {
        &self.declaration
    }
}

impl Implementable for Enum {
    fn trait_impls(&self) -> &[Implementation] {
        &self.trait_impl
    }

    fn auto_impls(&self) -> &[Implementation] {
        &self.auto_impl
    }

    fn blanket_impls(&self) -> &[Implementation] {
        &self.blanket
    }
}

impl Implementable for Primitive {
    fn trait_impls(&self) -> &[Implementation] {
        &self.trait_impl
    }

    fn auto_impls(&self) -> &[Implementation] {
        &self.auto_impl
    }

    fn blanket_impls(&self) -> &[Implementation] {
        &self.blanket
    }
}

pub(crate) fn parse_document(input: &NodeRef) -> Option<Document> {
    let main = input.select_first("#main").ok()?;

    let fqn = main.as_node().select_first(".fqn").ok()?;
    let title = fqn.as_node().last_child()?.text_contents();

    let after_title = fqn.as_node().next_sibling().and_then(skip_toggle_wrapper);
    match title.split_ascii_whitespace().next()? {
        "Crate" => {
            let (mark, description, module) = after_title.and_then(parse_module).unwrap_or_default();
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Crate(module)
            })
        }
        "Module" => {
            let (mark, description, module) = after_title.and_then(parse_module).unwrap_or_default();
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Module(module)
            })
        }
        "Constant" => {
            let (mark, description, declaration) = after_title.and_then(parse_declared)?;
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Constant(declaration)
            })
        }
        "Function" => {
            let (mark, description, declaration) = after_title.and_then(parse_declared)?;
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Function(declaration)
            })
        }
        "Macro" => {
            let (mark, description, declaration) = after_title.and_then(parse_declared)?;
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Macro(declaration)
            })
        }
        "Attribute" => {
            let (mark, description, declaration) = after_title.and_then(parse_declared)?;
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Attribute(declaration)
            })
        }
        "Type" => {
            let (mark, description, declaration) = after_title.and_then(parse_declared)?;
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Type(declaration)
            })
        }
        "Keyword" => {
            let (mark, description) = after_title.and_then(parse_basic).unwrap_or_default();
            Some(Document {
                title,
                mark,
                description,
                kind: DocumentKind::Keyword
            })
        }
        _ => return None
    }
}

fn parse_module(after_title: NodeRef) -> Option<(Mark, Vec<Section>, Module)> {
    let (head, mark) = if let Some(head) = skip_toggle_wrapper(after_title) {
        parse_marks_forward(head)
    } else {
        (None, Mark::default())
    };

    let head = head.and_then(skip_toggle_wrapper);
    let description = if let Some(head) = &head {
        parse_docblock(head)?
    } else {
        Vec::new()
    };

    let head = head.and_then(|head| head.next_sibling()).and_then(skip_toggle_wrapper);

    let (mut head, re_exports) = if let Some(head) = head {
        parse_exports_forward(head)?
    } else {
        (None, Vec::new())
    };

    let mut sub_item = Vec::new();
    while let Some(summary_head) = head {
        let (new_head, summary) = parse_summary_forward(summary_head)?;
        sub_item.push(summary);
        head = new_head.and_then(skip_toggle_wrapper);
    }

    Some((
        mark,
        description,
        Module {
            re_exports,
            sub_item
        }
    ))
}

fn parse_declared(after_title: NodeRef) -> Option<(Mark, Vec<Section>, Code)> {
    let code = parse_generic_code(&after_title);
    
    let head = after_title.next_sibling().and_then(skip_toggle_wrapper);

    let (head, mark) = if let Some(head) = head {
        parse_marks_forward(head)
    } else {
        (None, Mark::default())
    };

    let description = if let Some(head) = &head {
        parse_docblock(head)?
    } else {
        Vec::new()
    };

    Some((
        mark,
        description,
        code
    ))
}

fn parse_basic(after_title: NodeRef) -> Option<(Mark, Vec<Section>)> {
    let (head, mark) = parse_marks_forward(after_title);

    let description = if let Some(head) = &head {
        parse_docblock(head)?
    } else {
        Vec::new()
    };

    Some((
        mark,
        description
    ))
}