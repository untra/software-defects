mod go;
mod interp;
pub mod ir;
mod javascript;
mod python;

use crate::model::Span;
use crate::state::{Certainty, OperationId, StateGraph};
use std::collections::{BTreeMap, BTreeSet};

pub use interp::{MAX_CALL_DEPTH, MAX_OBSERVE_DEPTH, MAX_REFS, MAX_STEPS, MAX_UNROLL};

pub const ANY_ELEMENT: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sharing {
    RepeatedElement,
    ShallowCopy,
    SharedDefault,
    ReturnedAlias,
    ReturnedInternal,
    Reslice,
}

impl Sharing {
    pub fn mechanism(self) -> &'static str {
        match self {
            Self::RepeatedElement => "repeated-element",
            Self::ShallowCopy => "shallow-copy-nested",
            Self::SharedDefault => "shared-default",
            Self::ReturnedAlias => "argument-returned-alias",
            Self::ReturnedInternal => "internal-state-escape",
            Self::Reslice => "append-within-capacity",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    pub op: OperationId,
    pub kind: Sharing,
    pub qualifier: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Root {
    Binding(Span),
    Allocation(OperationId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Segment {
    Field(String),
    Element(Option<u64>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawPath {
    pub root: Root,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot {
    Field(String),
    Element(u64),
    AnyField,
    AnyElement,
}

#[derive(Debug, Clone)]
pub struct RawContext {
    pub parent: Option<usize>,
    pub call: OperationId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawSharing {
    pub op: OperationId,
    pub allocation: OperationId,
    pub kind: Sharing,
    pub qualifier: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RawBoundary {
    pub op: OperationId,
    pub kind: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone)]
pub struct WriteFact {
    pub op: OperationId,
    pub object: OperationId,
    pub slot: Slot,
    pub events: BTreeSet<Event>,
    pub certainty: Certainty,
    pub append: bool,
    pub boundary: Option<usize>,
    pub context: usize,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub write: usize,
    pub read: OperationId,
    pub events: BTreeSet<Event>,
    pub certainty: Certainty,
    pub segments: Vec<Segment>,
    pub boundary: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Requirement {
    pub kind: ir::RequirementKind,
    pub declaration: Span,
}

#[derive(Debug, Clone)]
pub struct ReadFact {
    pub range: Span,
    pub object: OperationId,
    pub property: Option<String>,
    pub context: usize,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub object: OperationId,
    pub range: Span,
}

#[derive(Debug, Clone, Default)]
pub struct HeapFacts {
    pub language: Option<ir::Lang>,
    pub ops: BTreeSet<OperationId>,
    pub paths: BTreeMap<OperationId, RawPath>,
    pub contexts: Vec<RawContext>,
    pub op_context: BTreeMap<OperationId, usize>,
    pub sharing: BTreeSet<RawSharing>,
    pub boundaries: Vec<RawBoundary>,
    pub writes: Vec<WriteFact>,
    pub observations: Vec<Observation>,
    pub requirements: BTreeMap<OperationId, Vec<Requirement>>,
    pub guarded: BTreeSet<OperationId>,
    pub modeled: Vec<Span>,
    pub bindings: Vec<Binding>,
    pub reads: Vec<ReadFact>,
    pub allocation_ranges: BTreeMap<OperationId, Span>,
    pub containers: BTreeMap<OperationId, ir::Container>,
}

impl HeapFacts {
    pub fn context_of(&self, op: OperationId) -> usize {
        self.op_context.get(&op).copied().unwrap_or(0)
    }
}

pub fn lower(
    language: &str,
    root: tree_sitter::Node<'_>,
    source: &str,
    uri: &str,
    resolve: &dyn Fn(&str, Span) -> Option<Span>,
    graph: &mut StateGraph,
) {
    let (lang, program) = match language {
        "javascript" | "typescript" => (
            ir::Lang::JavaScript,
            javascript::lower(root, source, resolve),
        ),
        "python" => (ir::Lang::Python, python::lower(root, source)),
        "go" => (ir::Lang::Go, go::lower(root, source, resolve)),
        _ => return,
    };
    graph.heap.language = Some(lang);
    interp::Interp::new(graph, uri, source, lang).run(&program);
}
