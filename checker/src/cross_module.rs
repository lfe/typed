use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::adt::{self, AdtDef};
use crate::error::{CheckError, Position};
use crate::sexp::types::*;
use crate::sexp::Parser;
use crate::typed_surface;

pub struct ProjectRegistry {
    pub modules: HashMap<String, Vec<AdtDef>>,
}

pub struct ImportedType {
    pub local_name: String,
    pub module: String,
    pub type_name: String,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn resolve_qualified(&self, qualified: &str) -> Option<&AdtDef> {
        let (mod_name, type_name) = qualified.split_once(':')?;
        let types = self.modules.get(mod_name)?;
        types.iter().find(|t| t.name == type_name)
    }

    pub fn module_exists(&self, mod_name: &str) -> bool {
        self.modules.contains_key(mod_name)
    }

    pub fn type_in_module(&self, mod_name: &str, type_name: &str) -> Option<&AdtDef> {
        self.modules
            .get(mod_name)?
            .iter()
            .find(|t| t.name == type_name)
    }
}

pub fn scan_project(input_file: &str) -> ProjectRegistry {
    let input_path = Path::new(input_file);
    let input_abs = std::fs::canonicalize(input_path).unwrap_or_else(|_| input_path.to_path_buf());
    let scan_dir = input_abs.parent().unwrap_or(&input_abs);

    let mut registry = ProjectRegistry::new();
    let tlfe_files = find_tlfe_files(scan_dir);

    for file_path in &tlfe_files {
        if let Ok(forms) = Parser::parse_all_file(file_path.to_str().unwrap_or("")) {
            let mut module_name = None;
            let mut types = Vec::new();

            for form in &forms {
                if let Some(mdef) = typed_surface::extract_module_def(form) {
                    module_name = Some(mdef.name);
                }
                if is_deftype(form) {
                    if let Ok(adt_def) = adt::extract_deftype(form) {
                        types.push(adt_def);
                    }
                }
                if is_defrecord_typed(form) {
                    if let Ok(adt_def) = adt::extract_defrecord(form) {
                        types.push(adt_def);
                    }
                }
            }

            if let Some(name) = module_name {
                if !types.is_empty() {
                    registry.modules.insert(name, types);
                }
            }
        }
    }

    registry
}

pub fn extract_import_types(form: &SExp) -> Option<Vec<ImportedType>> {
    let list = match form {
        SExp::List(l) => l,
        _ => return None,
    };
    if list.elements.is_empty() {
        return None;
    }
    match &list.elements[0] {
        SExp::Symbol(s) if s.value == "import-types" => {}
        _ => return None,
    }

    let mut imports = Vec::new();
    for clause in &list.elements[1..] {
        if let SExp::List(cl) = clause {
            if cl.elements.len() >= 3 {
                if let SExp::Symbol(from) = &cl.elements[0] {
                    if from.value == "from" {
                        if let SExp::Symbol(mod_sym) = &cl.elements[1] {
                            let mod_name = mod_sym.value.clone();
                            if let SExp::List(types_list) = &cl.elements[2] {
                                for type_elem in &types_list.elements {
                                    if let SExp::Symbol(ts) = type_elem {
                                        imports.push(ImportedType {
                                            local_name: ts.value.clone(),
                                            module: mod_name.clone(),
                                            type_name: ts.value.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if imports.is_empty() {
        None
    } else {
        Some(imports)
    }
}

#[allow(dead_code)]
pub fn resolve_type_name(
    type_str: &str,
    imports: &[ImportedType],
    registry: &ProjectRegistry,
    current_module: &str,
    file: &str,
    pos: Position,
) -> Result<String, CheckError> {
    if type_str.contains(':') {
        let (mod_name, type_name) = type_str.split_once(':').unwrap();
        if !registry.module_exists(mod_name) && mod_name != current_module {
            return Err(CheckError::Diagnostic {
                file: file.to_string(),
                pos,
                message: format!(
                    "unknown module `{}`; no `.tlfe` file declares module `{}`",
                    mod_name, mod_name
                ),
            });
        }
        if registry.type_in_module(mod_name, type_name).is_none() {
            return Err(CheckError::Diagnostic {
                file: file.to_string(),
                pos,
                message: format!("unknown type `{}` in module `{}`", type_name, mod_name),
            });
        }
        return Ok(type_str.to_string());
    }

    for imp in imports {
        if imp.local_name == type_str {
            return Ok(format!("{}:{}", imp.module, imp.type_name));
        }
    }

    Ok(type_str.to_string())
}

pub fn validate_imports(
    imports: &[ImportedType],
    registry: &ProjectRegistry,
    file: &str,
    pos: Position,
) -> Vec<CheckError> {
    let mut errors = Vec::new();
    for imp in imports {
        if !registry.module_exists(&imp.module) {
            errors.push(CheckError::Diagnostic {
                file: file.to_string(),
                pos,
                message: format!(
                    "unknown module `{}`; no `.tlfe` file declares module `{}`",
                    imp.module, imp.module
                ),
            });
        } else if registry
            .type_in_module(&imp.module, &imp.type_name)
            .is_none()
        {
            errors.push(CheckError::Diagnostic {
                file: file.to_string(),
                pos,
                message: format!(
                    "unknown type `{}` in module `{}`",
                    imp.type_name, imp.module
                ),
            });
        }
    }
    errors
}

#[allow(dead_code)]
fn find_project_dir(input_path: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(input_path).unwrap_or_else(|_| input_path.to_path_buf());
    let dir = abs.parent().unwrap_or(&abs);

    let mut current = dir.to_path_buf();
    loop {
        if current.join("rebar.config").exists()
            || current.join("Makefile").exists()
            || current.join("Cargo.toml").exists()
        {
            return current;
        }
        if !current.pop() {
            return dir.to_path_buf();
        }
    }
}

fn find_tlfe_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_tlfe_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_tlfe_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "_build" || name == "target" || name == ".git" || name == "node_modules"
                {
                    continue;
                }
                collect_tlfe_recursive(&path, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("tlfe") {
                files.push(path);
            }
        }
    }
}

fn is_deftype(form: &SExp) -> bool {
    matches!(form, SExp::List(l)
        if !l.elements.is_empty()
            && matches!(&l.elements[0], SExp::Symbol(s) if s.value == "deftype"))
}

fn is_defrecord_typed(form: &SExp) -> bool {
    matches!(form, SExp::List(l)
        if !l.elements.is_empty()
            && matches!(&l.elements[0], SExp::Symbol(s) if s.value == "defrecord/typed"))
}
