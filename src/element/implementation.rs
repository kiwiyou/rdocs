use super::{Code, SimpleItem};

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