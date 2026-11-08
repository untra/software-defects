use crate::model::{hash, DocumentReport, Span};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

pub type OperationId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Read,
    Write,
    Allocation,
    Call,
    Return,
    ExceptionalExit,
    Guard,
    Suspension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub uri: String,
    pub range: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub identity: String,
    pub binding_id: Option<String>,
    pub allocation_id: Option<String>,
    pub block_id: Option<String>,
    pub access_path: Option<String>,
    pub call_context: Option<String>,
    pub required_conditions: Vec<String>,
    pub evidence: Vec<String>,
    pub id: OperationId,
    pub kind: OperationKind,
    pub location: Location,
    pub binding: Option<Span>,
    pub guards: Vec<OperationId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    ReachingAssignment,
    ValueDependency,
    GovernedBy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: OperationId,
    pub to: OperationId,
    pub relationship: Relationship,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateGraph {
    pub contract_version: u32,
    pub snapshot_id: String,
    pub generation: u64,
    pub documents: Vec<DocumentIdentity>,
    pub bindings: Vec<BindingIdentity>,
    pub allocations: Vec<AllocationIdentity>,
    pub blocks: Vec<BasicBlock>,
    pub conditions: Vec<Condition>,
    pub access_paths: Vec<AccessPath>,
    pub call_contexts: Vec<CallContext>,
    pub evidence: Vec<SourceEvidence>,
    pub control_flow: Vec<ControlFlowEdge>,
    pub obligations: Vec<Obligation>,
    pub source_hash: String,
    pub operations: Vec<Operation>,
    pub edges: Vec<Edge>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    #[default]
    CandidateRelationship,
    FeasibleUnderModel,
    Reproduced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateTrace {
    pub contract_version: u32,
    pub snapshot_id: String,
    pub origin_operations: Vec<String>,
    pub invalidation_status: InvalidationStatus,
    pub invalidating_operations: Vec<String>,
    pub affected_operations: Vec<String>,
    pub supporting_evidence: Vec<String>,
    pub required_conditions: Vec<String>,
    #[serde(default)]
    pub evidence_level: EvidenceLevel,
    pub source_hash: String,
    pub origins: Vec<Location>,
    pub invalidations: Vec<Location>,
    pub affected_uses: Vec<Location>,
    pub required_condition: Option<String>,
    pub operations: Vec<Operation>,
    pub edges: Vec<Edge>,
    pub assumptions: Vec<String>,
}

type Definitions = BTreeMap<Span, BTreeSet<OperationId>>;

fn children(n: Node<'_>) -> Vec<Node<'_>> {
    n.named_children(&mut n.walk()).collect()
}

struct Builder<'a, F> {
    graph: StateGraph,
    uri: &'a str,
    source: &'a str,
    resolve: F,
    terminated: bool,
}

impl<F: Fn(&str, Span) -> Option<Span>> Builder<'_, F> {
    fn add(&mut self, n: Node<'_>, kind: OperationKind, guards: &[OperationId]) -> OperationId {
        let id = self.graph.operations.len();
        let range = Span::of(n);
        let binding = (self.resolve)(&self.source[n.byte_range()], range);
        self.graph.operations.push(Operation {
            identity: String::new(),
            binding_id: None,
            allocation_id: None,
            block_id: None,
            access_path: None,
            call_context: None,
            required_conditions: vec![],
            evidence: vec![],
            id,
            kind,
            location: Location {
                uri: self.uri.into(),
                range,
            },
            binding,
            guards: guards.to_vec(),
        });
        for &guard in guards {
            self.edge(guard, id, Relationship::GovernedBy);
        }
        id
    }

    fn edge(&mut self, from: OperationId, to: OperationId, relationship: Relationship) {
        self.graph.edges.push(Edge {
            from,
            to,
            relationship,
        });
    }

    fn visit(
        &mut self,
        n: Node<'_>,
        defs: &mut Definitions,
        guards: &[OperationId],
    ) -> Vec<OperationId> {
        let kind = n.kind();
        if self.terminated {
            return vec![];
        }
        if matches!(
            kind,
            "function_declaration"
                | "function_definition"
                | "function_item"
                | "method_definition"
                | "arrow_function"
                | "function_expression"
                | "func_literal"
                | "lambda"
        ) {
            let mut local = Definitions::new();
            let outer_terminated = self.terminated;
            self.terminated = false;
            for child in children(n) {
                self.visit(child, &mut local, &[]);
            }
            self.terminated = outer_terminated;
            return vec![];
        }
        if matches!(kind, "if_statement" | "if_expression") {
            if let Some(condition) = n.child_by_field_name("condition") {
                let inputs = self.visit(condition, defs, guards);
                let guard = self.add(condition, OperationKind::Guard, guards);
                for input in inputs {
                    self.edge(input, guard, Relationship::ValueDependency);
                }
                let mut nested = guards.to_vec();
                nested.push(guard);
                let mut yes = defs.clone();
                let mut no = defs.clone();
                if let Some(body) = n
                    .child_by_field_name("consequence")
                    .or_else(|| n.child_by_field_name("body"))
                {
                    self.visit(body, &mut yes, &nested);
                }
                let yes_terminated = self.terminated;
                self.terminated = false;
                if let Some(body) = n.child_by_field_name("alternative") {
                    self.visit(body, &mut no, &nested);
                }
                let no_terminated = self.terminated;
                if yes_terminated {
                    yes.clear();
                }
                if !no_terminated {
                    for (binding, values) in no {
                        yes.entry(binding).or_default().extend(values);
                    }
                }
                self.terminated = yes_terminated && no_terminated;
                *defs = yes;
                return vec![];
            }
        }
        if matches!(
            kind,
            "for_statement"
                | "for_expression"
                | "while_statement"
                | "while_expression"
                | "loop_expression"
        ) {
            let entry = defs.clone();
            for child in children(n) {
                self.visit(child, defs, guards);
            }
            self.terminated = false;
            for (binding, values) in entry {
                defs.entry(binding).or_default().extend(values);
            }
            self.graph.limitations.push("Loop backedges are not resolved; reaching assignments are incomplete inside loops.".into());
            return vec![];
        }
        if matches!(kind, "try_statement" | "try_expression") {
            let entry = defs.clone();
            for child in children(n) {
                self.terminated = false;
                self.visit(child, defs, guards);
            }
            self.terminated = false;
            for (binding, values) in entry {
                defs.entry(binding).or_default().extend(values);
            }
            self.graph
                .limitations
                .push("Exceptional control flow and finally completion are unresolved.".into());
            return vec![];
        }
        let left = n
            .child_by_field_name("left")
            .or_else(|| n.child_by_field_name("name"))
            .or_else(|| n.child_by_field_name("pattern"));
        let right = n
            .child_by_field_name("right")
            .or_else(|| n.child_by_field_name("value"));
        if matches!(
            kind,
            "assignment"
                | "assignment_expression"
                | "assignment_statement"
                | "variable_declarator"
                | "let_declaration"
                | "short_var_declaration"
                | "var_spec"
                | "augmented_assignment"
                | "augmented_assignment_expression"
                | "compound_assignment_expr"
        ) {
            if let (Some(left), Some(right)) = (left, right) {
                let mut inputs = self.visit(right, defs, guards);
                if matches!(
                    kind,
                    "augmented_assignment"
                        | "augmented_assignment_expression"
                        | "compound_assignment_expr"
                ) {
                    inputs.extend(self.visit(left, defs, guards));
                }
                let targets = if left.kind() == "expression_list" {
                    children(left)
                } else {
                    vec![left]
                };
                for target in targets {
                    let id = self.add(target, OperationKind::Write, guards);
                    for &input in &inputs {
                        self.edge(input, id, Relationship::ValueDependency);
                    }
                    if let Some(binding) = self.graph.operations[id].binding {
                        defs.insert(binding, BTreeSet::from([id]));
                    } else {
                        self.graph.limitations.push(
                            "Property writes and destructuring require object identity resolution."
                                .into(),
                        );
                    }
                }
                return inputs;
            }
        }
        if matches!(
            kind,
            "update_expression" | "inc_statement" | "dec_statement"
        ) {
            if let Some(target) = n
                .child_by_field_name("argument")
                .or_else(|| children(n).first().copied())
            {
                let inputs = self.visit(target, defs, guards);
                let id = self.add(target, OperationKind::Write, guards);
                for input in inputs {
                    self.edge(input, id, Relationship::ValueDependency);
                }
                if let Some(binding) = self.graph.operations[id].binding {
                    defs.insert(binding, BTreeSet::from([id]));
                }
                return vec![id];
            }
        }
        if kind == "identifier" {
            if n.parent().is_some_and(|parent| {
                matches!(parent.kind(), "member_expression" | "attribute")
                    && parent.child_by_field_name("object") != Some(n)
            }) {
                return vec![];
            }
            let id = self.add(n, OperationKind::Read, guards);
            if let Some(binding) = self.graph.operations[id].binding {
                if let Some(origins) = defs.get(&binding) {
                    for &origin in origins {
                        self.edge(origin, id, Relationship::ReachingAssignment);
                    }
                }
            }
            return vec![id];
        }
        let mut inputs = vec![];
        for child in children(n) {
            inputs.extend(self.visit(child, defs, guards));
        }
        let operation = match kind {
            "call" | "call_expression" => Some(OperationKind::Call),
            "return_statement" | "return_expression" => Some(OperationKind::Return),
            "raise_statement" | "throw_statement" => Some(OperationKind::ExceptionalExit),
            "await" | "await_expression" | "yield" | "yield_expression" => {
                Some(OperationKind::Suspension)
            }
            "object" | "dictionary" | "list" | "array" | "array_expression" => {
                Some(OperationKind::Allocation)
            }
            _ => None,
        };
        if let Some(kind) = operation {
            let id = self.add(n, kind, guards);
            for input in inputs {
                self.edge(input, id, Relationship::ValueDependency);
            }
            if matches!(kind, OperationKind::Return | OperationKind::ExceptionalExit) {
                self.terminated = true;
            }
            if kind == OperationKind::Call {
                self.graph.limitations.push(
                    "Call effects and returned aliases are not resolved in this graph.".into(),
                );
            }
            return vec![id];
        }
        inputs
    }
}

pub fn extract(
    root: Node<'_>,
    uri: &str,
    source: &str,
    resolve: impl Fn(&str, Span) -> Option<Span>,
) -> StateGraph {
    let mut builder = Builder {
        graph: StateGraph {
            source_hash: hash(source),
            ..StateGraph::default()
        },
        uri,
        source,
        resolve,
        terminated: false,
    };
    builder.visit(root, &mut Definitions::new(), &[]);
    builder.graph.limitations.push("Candidate dependencies only: branch polarity, exceptional control flow, closures, aliases and scheduling feasibility are not established.".into());
    builder.graph.limitations.sort();
    builder.graph.limitations.dedup();
    builder.graph.qualify(
        0,
        vec![DocumentIdentity {
            id: uri.into(),
            uri: uri.into(),
            source_hash: hash(source),
            byte_length: source.len(),
            version: None,
        }],
    );
    builder.graph
}

impl StateGraph {
    pub fn candidate_region(
        &self,
        origin: OperationId,
        affected_use: OperationId,
    ) -> BTreeSet<OperationId> {
        fn reachable(edges: &[Edge], start: usize, reverse: bool) -> BTreeSet<usize> {
            let mut seen = BTreeSet::from([start]);
            let mut work = vec![start];
            while let Some(at) = work.pop() {
                for edge in edges {
                    let (from, to) = if reverse {
                        (edge.to, edge.from)
                    } else {
                        (edge.from, edge.to)
                    };
                    if from == at && seen.insert(to) {
                        work.push(to);
                    }
                }
            }
            seen
        }
        reachable(&self.edges, origin, false)
            .intersection(&reachable(&self.edges, affected_use, true))
            .copied()
            .collect()
    }

    pub fn explain(&mut self, report: &mut DocumentReport) {
        for finding in &report.findings {
            if finding.origin != "direct" || finding.defect != "divide-by-zero" {
                continue;
            }
            for op in &mut self.operations {
                if op.kind != OperationKind::Read || !finding.trigger.contains(op.location.range) {
                    continue;
                }
                let id = format!("{}:condition:nonzero", op.identity);
                if !self.conditions.iter().any(|condition| condition.id == id) {
                    self.conditions.push(Condition {
                        id: id.clone(),
                        predicate: Predicate::Nonzero {
                            value: op.identity.clone(),
                        },
                        uncertainty: vec![
                            "Numeric type and path feasibility are unresolved.".into()
                        ],
                        evidence: op.evidence.clone(),
                    });
                }
                if !op.required_conditions.contains(&id) {
                    op.required_conditions.push(id);
                }
            }
        }
        for finding in &mut report.findings {
            if finding.origin != "direct" {
                continue;
            }
            let uses: Vec<_> = self
                .operations
                .iter()
                .filter(|op| {
                    op.kind == OperationKind::Read && finding.trigger.contains(op.location.range)
                })
                .collect();
            if uses.is_empty() {
                continue;
            }
            let mut ids = BTreeSet::new();
            let mut origins = vec![];
            for affected in &uses {
                for edge in self.edges.iter().filter(|edge| {
                    edge.to == affected.id && edge.relationship == Relationship::ReachingAssignment
                }) {
                    origins.push(self.operations[edge.from].location.clone());
                    ids.extend(self.candidate_region(edge.from, affected.id));
                }
                ids.insert(affected.id);
            }
            let mut pending: Vec<_> = ids.iter().copied().collect();
            while let Some(id) = pending.pop() {
                for edge in self.edges.iter().filter(|edge| edge.to == id) {
                    if ids.insert(edge.from) {
                        pending.push(edge.from);
                    }
                }
            }
            finding.state_trace = Some(StateTrace {
                contract_version: CONTRACT_VERSION,
                snapshot_id: self.snapshot_id.clone(),
                origin_operations: self
                    .operations
                    .iter()
                    .filter(|op| {
                        origins
                            .iter()
                            .any(|loc| loc.uri == op.location.uri && loc.range == op.location.range)
                    })
                    .map(|op| op.identity.clone())
                    .collect(),
                invalidation_status: InvalidationStatus::Unknown,
                invalidating_operations: vec![],
                affected_operations: uses.iter().map(|op| op.identity.clone()).collect(),
                supporting_evidence: ids
                    .iter()
                    .flat_map(|id| self.operations[*id].evidence.clone())
                    .collect(),
                required_conditions: uses
                    .iter()
                    .flat_map(|op| op.required_conditions.clone())
                    .collect(),
                evidence_level: EvidenceLevel::CandidateRelationship,
                source_hash: self.source_hash.clone(),
                origins,
                invalidations: vec![],
                affected_uses: uses.iter().map(|op| op.location.clone()).collect(),
                required_condition: (finding.defect == "divide-by-zero")
                    .then(|| "Denominator must be nonzero at this operation.".into()),
                operations: self
                    .operations
                    .iter()
                    .filter(|op| ids.contains(&op.id))
                    .cloned()
                    .collect(),
                edges: self
                    .edges
                    .iter()
                    .filter(|edge| ids.contains(&edge.from) && ids.contains(&edge.to))
                    .cloned()
                    .collect(),
                assumptions: self.limitations.clone(),
            });
        }
    }
}

pub use crate::state_contract::*;
pub const CONTRACT_VERSION: u32 = 2;

impl StateGraph {
    pub(crate) fn qualify(&mut self, generation: u64, mut documents: Vec<DocumentIdentity>) {
        documents.sort_by(|a, b| a.id.cmp(&b.id));
        self.contract_version = CONTRACT_VERSION;
        self.generation = generation;
        self.snapshot_id = hash(
            &serde_json::to_string(
                &serde_json::to_value((generation, &documents)).expect("snapshot value"),
            )
            .expect("snapshot serialization"),
        );
        self.qualify_snapshot(generation, documents, self.snapshot_id.clone());
    }

    pub(crate) fn qualify_snapshot(
        &mut self,
        generation: u64,
        documents: Vec<DocumentIdentity>,
        snapshot_id: String,
    ) {
        self.contract_version = CONTRACT_VERSION;
        self.generation = generation;
        self.snapshot_id = snapshot_id;
        self.documents = documents;
        self.bindings.clear();
        self.allocations.clear();
        self.evidence.clear();
        let mut bindings = BTreeMap::new();
        for op in &mut self.operations {
            let prefix = format!("{}:{}", self.snapshot_id, op.location.uri);
            op.identity = format!("{prefix}:operation:{}", op.id);
            op.binding_id = op.binding.map(|range| {
                let id = format!("{prefix}:binding:{}:{}", range.start, range.end);
                bindings.insert(
                    id.clone(),
                    Location {
                        uri: op.location.uri.clone(),
                        range,
                    },
                );
                id
            });
            if op.kind == OperationKind::Allocation {
                let id = format!("{prefix}:allocation:{}", op.id);
                self.allocations.push(AllocationIdentity {
                    id: id.clone(),
                    operation: op.identity.clone(),
                });
                op.allocation_id = Some(id);
            }
            let id = format!("{prefix}:evidence:{}", op.id);
            self.evidence.push(SourceEvidence {
                id: id.clone(),
                document: op.location.uri.clone(),
                source_hash: self
                    .documents
                    .iter()
                    .find(|d| d.uri == op.location.uri)
                    .expect("operation document")
                    .source_hash
                    .clone(),
                range: op.location.range,
            });
            op.evidence = vec![id];
        }
        self.bindings = bindings
            .into_iter()
            .map(|(id, location)| BindingIdentity { id, location })
            .collect();
    }
}

impl StateGraph {
    pub fn validate(&self) -> Result<(), String> {
        if self.contract_version != CONTRACT_VERSION {
            return Err("unsupported graph version".into());
        }
        let expected = hash(
            &serde_json::to_string(
                &serde_json::to_value((self.generation, &self.documents))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?,
        );
        if self.snapshot_id != expected {
            return Err("snapshot identity mismatch".into());
        }
        let documents: BTreeMap<_, _> =
            self.documents.iter().map(|d| (d.uri.as_str(), d)).collect();
        if documents.len() != self.documents.len()
            || self
                .documents
                .iter()
                .map(|d| &d.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.documents.len()
        {
            return Err("duplicate document identity".into());
        }
        if !self
            .documents
            .iter()
            .any(|document| document.source_hash == self.source_hash)
        {
            return Err("mixed source snapshot".into());
        }
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        let kinds = [
            "operations",
            "bindings",
            "allocations",
            "blocks",
            "conditions",
            "access_paths",
            "call_contexts",
            "evidence",
            "obligations",
        ];
        let mut identities = BTreeMap::new();
        for kind in kinds {
            for item in value[kind].as_array().ok_or("missing collection")? {
                let id = item[if kind == "operations" {
                    "identity"
                } else {
                    "id"
                }]
                .as_str()
                .ok_or("missing identity")?;
                if !id.starts_with(&format!("{}:", self.snapshot_id)) {
                    return Err("mixed snapshot identity".into());
                }
                if identities.insert(id, kind).is_some() {
                    return Err("duplicate graph identity".into());
                }
            }
        }
        let reference = |id: &str, kinds: &[&str]| -> Result<(), String> {
            if identities.get(id).is_some_and(|kind| kinds.contains(kind)) {
                Ok(())
            } else {
                Err(format!("dangling or wrong-kind reference: {id}"))
            }
        };
        let span = |uri: &str, range: Span| -> Result<(), String> {
            let doc = documents.get(uri).ok_or("unknown document")?;
            if range.start > range.end || range.end > doc.byte_length {
                return Err("invalid source span".into());
            }
            Ok(())
        };
        let operations: BTreeMap<_, _> = self.operations.iter().map(|op| (op.id, op)).collect();
        if operations.len() != self.operations.len() {
            return Err("duplicate operation index".into());
        }
        for op in &self.operations {
            span(&op.location.uri, op.location.range)?;
            if let Some(range) = op.binding {
                span(&op.location.uri, range)?;
            }
            for guard in &op.guards {
                if !operations
                    .get(guard)
                    .is_some_and(|op| op.kind == OperationKind::Guard)
                {
                    return Err("dangling guard".into());
                }
            }
            for (id, kind) in [
                (&op.binding_id, "bindings"),
                (&op.allocation_id, "allocations"),
                (&op.block_id, "blocks"),
                (&op.access_path, "access_paths"),
                (&op.call_context, "call_contexts"),
            ] {
                if let Some(id) = id {
                    reference(id, &[kind])?;
                }
            }
        }
        for edge in &self.edges {
            if !operations.contains_key(&edge.from) || !operations.contains_key(&edge.to) {
                return Err("dangling dependency edge".into());
            }
        }
        for binding in &self.bindings {
            span(&binding.location.uri, binding.location.range)?;
        }
        for evidence in &self.evidence {
            let doc = self
                .documents
                .iter()
                .find(|d| d.id == evidence.document)
                .ok_or("unknown evidence document")?;
            if doc.source_hash != evidence.source_hash {
                return Err("stale evidence".into());
            }
            span(&doc.uri, evidence.range)?;
        }
        for kind in kinds {
            for item in value[kind].as_array().ok_or("missing collection")? {
                for (field, targets) in [
                    ("operation", vec!["operations"]),
                    ("call", vec!["operations"]),
                    ("parent", vec!["call_contexts"]),
                    ("condition", vec!["conditions"]),
                    ("root", vec!["bindings", "allocations"]),
                ] {
                    if let Some(id) = item[field].as_str() {
                        reference(id, &targets)?;
                    }
                }
                for (field, target) in [
                    ("evidence", "evidence"),
                    ("operations", "operations"),
                    ("required_conditions", "conditions"),
                ] {
                    if let Some(ids) = item[field].as_array() {
                        for id in ids {
                            reference(id.as_str().ok_or("invalid reference")?, &[target])?;
                        }
                    }
                }
            }
        }
        for edge in &self.control_flow {
            reference(&edge.from, &["blocks"])?;
            reference(&edge.to, &["blocks"])?;
            if let Some(id) = &edge.condition {
                reference(id, &["conditions"])?;
            }
            for id in &edge.evidence {
                reference(id, &["evidence"])?;
            }
        }
        for condition in &self.conditions {
            let predicate =
                serde_json::to_value(&condition.predicate).map_err(|e| e.to_string())?;
            if let Some(ids) = predicate["conditions"].as_array() {
                for id in ids {
                    reference(id.as_str().ok_or("invalid condition")?, &["conditions"])?;
                }
            }
            for field in [
                "value",
                "index",
                "collection",
                "resource",
                "owner",
                "location",
                "left",
                "right",
            ] {
                if let Some(id) = predicate[field].as_str() {
                    reference(
                        id,
                        &["operations", "access_paths", "bindings", "allocations"],
                    )?;
                }
            }
        }
        for path in &self.access_paths {
            for segment in &path.segments {
                if let PathSegment::Index { value } = segment {
                    reference(value, &["operations", "access_paths"])?;
                }
            }
        }
        for context in &self.call_contexts {
            let mut seen = BTreeSet::from([context.id.as_str()]);
            let mut parent = context.parent.as_deref();
            while let Some(id) = parent {
                if !seen.insert(id) {
                    return Err("cyclic call context".into());
                }
                parent = self
                    .call_contexts
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or("dangling call context")?
                    .parent
                    .as_deref();
            }
        }
        for obligation in &self.obligations {
            if obligation.outcome == Outcome::ViolatedOnDemonstratedPath
                && obligation.evidence.is_empty()
            {
                return Err("demonstrated violation requires evidence".into());
            }
        }
        Ok(())
    }
}

impl StateTrace {
    pub fn validate(&self, graph: &StateGraph) -> Result<(), String> {
        graph.validate()?;
        if self.contract_version != CONTRACT_VERSION
            || self.snapshot_id != graph.snapshot_id
            || self.source_hash != graph.source_hash
        {
            return Err("unsupported or stale trace".into());
        }
        let operations: BTreeMap<_, _> = graph
            .operations
            .iter()
            .map(|op| (&op.identity, op))
            .collect();
        for id in self
            .origin_operations
            .iter()
            .chain(&self.invalidating_operations)
            .chain(&self.affected_operations)
        {
            if !operations.contains_key(id) {
                return Err("dangling trace operation".into());
            }
        }
        if self.invalidation_status != InvalidationStatus::Invalidated
            && (!self.invalidations.is_empty() || !self.invalidating_operations.is_empty())
        {
            return Err("invented invalidation".into());
        }
        for id in &self.supporting_evidence {
            if !graph.evidence.iter().any(|e| &e.id == id) {
                return Err("dangling trace evidence".into());
            }
        }
        for id in &self.required_conditions {
            if !graph.conditions.iter().any(|c| &c.id == id) {
                return Err("dangling trace condition".into());
            }
        }
        let mut ids = BTreeSet::new();
        for op in &self.operations {
            if !ids.insert(op.id) {
                return Err("duplicate trace operation".into());
            }
            let original = operations
                .get(&op.identity)
                .ok_or("dangling trace operation")?;
            if serde_json::to_value(original).map_err(|e| e.to_string())?
                != serde_json::to_value(op).map_err(|e| e.to_string())?
            {
                return Err("inconsistent trace operation".into());
            }
        }
        for op in &self.operations {
            if op.guards.iter().any(|id| !ids.contains(id)) {
                return Err("dangling trace guard".into());
            }
        }
        for edge in &self.edges {
            if !ids.contains(&edge.from) || !ids.contains(&edge.to) {
                return Err("dangling trace edge".into());
            }
            if !graph.edges.iter().any(|original| {
                original.from == edge.from
                    && original.to == edge.to
                    && original.relationship == edge.relationship
            }) {
                return Err("inconsistent trace edge".into());
            }
        }
        for location in self
            .origins
            .iter()
            .chain(&self.invalidations)
            .chain(&self.affected_uses)
        {
            let document = graph
                .documents
                .iter()
                .find(|d| d.uri == location.uri)
                .ok_or("unknown trace document")?;
            if location.range.start > location.range.end
                || location.range.end > document.byte_length
            {
                return Err("invalid trace span".into());
            }
        }
        Ok(())
    }
}
