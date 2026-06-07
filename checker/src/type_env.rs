use std::collections::HashMap;

use crate::adt::AdtDef;

#[derive(Debug, Default)]
pub struct TypeEnv {
    types: HashMap<String, AdtDef>,
    ctor_to_type: HashMap<String, String>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, adt: AdtDef) {
        for ctor in &adt.constructors {
            self.ctor_to_type
                .insert(ctor.name.clone(), adt.name.clone());
        }
        self.types.insert(adt.name.clone(), adt);
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "used for cross-module type resolution in M4+")
    )]
    pub fn lookup_type(&self, name: &str) -> Option<&AdtDef> {
        self.types.get(name)
    }

    pub fn lookup_ctor(&self, ctor_name: &str) -> Option<&AdtDef> {
        self.ctor_to_type
            .get(ctor_name)
            .and_then(|type_name| self.types.get(type_name))
    }

    pub fn all_ctor_names(&self) -> Vec<String> {
        self.ctor_to_type.keys().cloned().collect()
    }

    pub fn all_types(&self) -> impl Iterator<Item = &AdtDef> {
        self.types.values()
    }
}
