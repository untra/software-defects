use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA_VERSION: u32 = 1;
pub const FAMILY: &str = "mutated-state";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn contains(self, other: Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
    pub fn len(self) -> usize {
        self.end - self.start
    }
    pub fn of(node: tree_sitter::Node<'_>) -> Self {
        Self {
            start: node.start_byte(),
            end: node.end_byte(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", content = "parameter", rename_all = "snake_case")]
pub enum Effect {
    LocalMutation,
    ArgumentMutation(usize),
    ReceiverMutation,
    GlobalMutation,
    ExternalRead,
    Nondeterminism,
    StructuralSignal,
}
impl Effect {
    pub fn observable(&self) -> bool {
        !matches!(self, Self::LocalMutation | Self::StructuralSignal)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Evidence {
    pub uri: String,
    pub range: Span,
    pub rule_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub family: String,
    pub defect: String,
    pub trigger: Span,
    pub function: Option<Span>,
    pub effect: Effect,
    pub origin: String,
    pub resolution: String,
    pub explanation: String,
    pub evidence: Vec<Evidence>,
    pub call_path: Vec<Evidence>,
    pub related_cwes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub range: Span,
    pub body: Span,
    pub parameters: Vec<String>,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unresolved {
    pub range: Span,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReport {
    pub uri: String,
    pub language: String,
    pub source_hash: String,
    pub version: Option<i64>,
    pub functions: Vec<Function>,
    pub findings: Vec<Finding>,
    pub unresolved: Vec<Unresolved>,
    pub parse_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub generation: u64,
    pub documents: Vec<DocumentReport>,
    pub limitations: Vec<String>,
}

pub fn hash(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

pub fn position(source: &str, byte: usize) -> Position {
    let prefix = &source[..byte];
    let start = prefix.rfind('\n').map_or(0, |i| i + 1);
    Position {
        line: prefix.bytes().filter(|b| *b == b'\n').count(),
        character: prefix[start..].encode_utf16().count(),
    }
}

pub fn byte_offset(source: &str, pos: Position) -> Option<usize> {
    let mut start = 0;
    for _ in 0..pos.line {
        start += source.get(start..)?.find('\n')? + 1;
    }
    let mut units = 0;
    for (offset, ch) in source[start..].char_indices() {
        if units == pos.character {
            return Some(start + offset);
        }
        if ch == '\n' || ch == '\r' {
            return None;
        }
        units += ch.len_utf16();
        if units > pos.character {
            return None;
        }
    }
    (units == pos.character).then_some(source.len())
}
