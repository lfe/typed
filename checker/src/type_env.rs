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

    pub fn lookup_record_accessor(&self, func_name: &str) -> Option<(&str, &str)> {
        for adt in self.types.values() {
            if adt.constructors.len() == 1 && adt.constructors[0].name == adt.name {
                let rec_name = &adt.name;
                let prefix = format!("{}-", rec_name);
                if let Some(field_name) = func_name.strip_prefix(&prefix) {
                    if let Some(field) = adt.constructors[0]
                        .fields
                        .iter()
                        .find(|f| f.name == field_name)
                    {
                        return Some((rec_name, &field.type_expr));
                    }
                }
            }
        }
        None
    }

    pub fn check_unknown_record_field(
        &self,
        func_name: &str,
    ) -> Option<(String, String, Vec<String>)> {
        for adt in self.types.values() {
            if adt.constructors.len() == 1 && adt.constructors[0].name == adt.name {
                let rec_name = &adt.name;
                let prefix = format!("{}-", rec_name);
                if let Some(field_name) = func_name.strip_prefix(&prefix) {
                    let known = adt.constructors[0]
                        .fields
                        .iter()
                        .any(|f| f.name == field_name);
                    if !known {
                        let available: Vec<String> = adt.constructors[0]
                            .fields
                            .iter()
                            .map(|f| f.name.clone())
                            .collect();
                        return Some((rec_name.clone(), field_name.to_string(), available));
                    }
                }
            }
        }
        None
    }
}
