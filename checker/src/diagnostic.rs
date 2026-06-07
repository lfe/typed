#![expect(
    dead_code,
    reason = "diagnostic engine wired incrementally; fully used by M2 close"
)]

use crate::error::{CheckError, Position};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub file: String,
    pub pos: Position,
    pub message: String,
    pub missing_ctors: Vec<String>,
    pub hint: Option<String>,
    pub source_line: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn add_check_error(&mut self, err: &CheckError, source: Option<&str>) {
        match err {
            CheckError::Diagnostic { file, pos, message } => {
                self.push(Diagnostic {
                    code: "E001".to_string(),
                    severity: Severity::Error,
                    file: file.clone(),
                    pos: *pos,
                    message: message.clone(),
                    missing_ctors: vec![],
                    hint: None,
                    source_line: source.and_then(|s| get_source_line(s, pos.line)),
                });
            }
            CheckError::NonExhaustive {
                file,
                pos,
                type_name,
                missing,
            } => {
                let hint = "add clauses for the missing constructor(s), or use `_` as a catch-all"
                    .to_string();
                self.push(Diagnostic {
                    code: "E100".to_string(),
                    severity: Severity::Error,
                    file: file.clone(),
                    pos: *pos,
                    message: format!("non-exhaustive pattern match on type `{}`", type_name),
                    missing_ctors: missing.clone(),
                    hint: Some(hint),
                    source_line: source.and_then(|s| get_source_line(s, pos.line)),
                });
            }
        }
    }

    pub fn add_check_error_as_warning(&mut self, err: &CheckError, source: Option<&str>) {
        match err {
            CheckError::Diagnostic { file, pos, message } => {
                self.push(Diagnostic {
                    code: "W001".to_string(),
                    severity: Severity::Warning,
                    file: file.clone(),
                    pos: *pos,
                    message: message.clone(),
                    missing_ctors: vec![],
                    hint: None,
                    source_line: source.and_then(|s| get_source_line(s, pos.line)),
                });
            }
            _ => self.add_check_error(err, source),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn render_human(&self) -> String {
        let mut out = String::new();
        for diag in &self.diagnostics {
            out.push_str(&render_one_human(diag));
            out.push('\n');
        }
        out
    }

    pub fn render_json(&self) -> String {
        let mut items = Vec::new();
        for diag in &self.diagnostics {
            items.push(render_one_json(diag));
        }
        format!("[{}]", items.join(","))
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

fn render_one_human(diag: &Diagnostic) -> String {
    let severity_label = match diag.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };

    let mut out = format!(
        "{}[{}]: {}\n  --> {}:{}:{}\n",
        severity_label, diag.code, diag.message, diag.file, diag.pos.line, diag.pos.column
    );

    if let Some(src_line) = &diag.source_line {
        let line_num = format!("{}", diag.pos.line);
        let padding = " ".repeat(line_num.len());
        out.push_str(&format!("   {} |\n", padding));
        out.push_str(&format!("   {} | {}\n", line_num, src_line));
        let col = if diag.pos.column > 0 {
            diag.pos.column - 1
        } else {
            0
        };
        out.push_str(&format!("   {} | {}^\n", padding, " ".repeat(col)));
    }

    if !diag.missing_ctors.is_empty() {
        out.push_str("   |\n   = These values are not matched:\n");
        for ctor in &diag.missing_ctors {
            out.push_str(&format!("       - {}\n", ctor));
        }
    }

    if let Some(hint) = &diag.hint {
        out.push_str(&format!("   = Hint: {}.\n", hint));
    }

    out
}

fn render_one_json(diag: &Diagnostic) -> String {
    let missing_json: Vec<String> = diag
        .missing_ctors
        .iter()
        .map(|c| format!("\"{}\"", escape_json(c)))
        .collect();
    let hint_json = match &diag.hint {
        Some(h) => format!("\"{}\"", escape_json(h)),
        None => "null".to_string(),
    };
    format!(
        concat!(
            "{{",
            "\"code\":\"{code}\",",
            "\"severity\":\"{severity}\",",
            "\"file\":\"{file}\",",
            "\"line\":{line},",
            "\"column\":{col},",
            "\"message\":\"{msg}\",",
            "\"missing_ctors\":[{missing}],",
            "\"hint\":{hint}",
            "}}"
        ),
        code = escape_json(&diag.code),
        severity = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        },
        file = escape_json(&diag.file),
        line = diag.pos.line,
        col = diag.pos.column,
        msg = escape_json(&diag.message),
        missing = missing_json.join(","),
        hint = hint_json,
    )
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn get_source_line(source: &str, line: usize) -> Option<String> {
    if line == 0 {
        return None;
    }
    source.lines().nth(line - 1).map(|s| s.to_string())
}
