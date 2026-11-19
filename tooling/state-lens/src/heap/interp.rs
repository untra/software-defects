use super::ir::{Container, Entry, Expr, Func, Index, Key, Lang, Param, RequirementKind, Stmt};
use super::{
    Binding, Event, Observation, RawBoundary, RawContext, RawPath, RawSharing, Requirement, Root,
    Segment, Sharing, Slot, WriteFact, ANY_ELEMENT,
};
use crate::model::Span;
use crate::state::{
    Certainty, Edge, Location, Operation, OperationId, OperationKind, Relationship, StateGraph,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::rc::Rc;

pub const MAX_STEPS: usize = 50_000;
pub const MAX_CALL_DEPTH: usize = 4;
pub const MAX_UNROLL: u64 = 8;
pub const MAX_OBSERVE_DEPTH: usize = 4;
pub const MAX_REFS: usize = 32;
const UNKNOWN_COUNT_UNROLL: u64 = 2;
const MAX_STANDALONE_PASSES: usize = 64;
const ENTRY_POINT: &str = "main";

const LIMIT_LOOPS: &str =
    "Loop bodies are interpreted once; heap facts inside loops are possible, not definite.";
const LIMIT_EXCEPTIONS: &str = "Exceptional completion is approximated by joining the pre-try state, throw points and normal completion.";
const LIMIT_BUDGET: &str =
    "Object-identity analysis budget was exhausted; later heap facts are omitted.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Use,
    Alias,
    Base,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Val {
    Obj {
        id: OperationId,
        pointer: bool,
    },
    Slice {
        backing: OperationId,
        offset: Option<u64>,
        length: Option<u64>,
        capacity: Option<u64>,
    },
    Int(i64),
    Func(usize),
    Null,
    Unknown(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ref {
    val: Val,
    events: BTreeSet<Event>,
}

impl Ref {
    fn plain(val: Val) -> Self {
        Self {
            val,
            events: BTreeSet::new(),
        }
    }
    fn object(&self) -> Option<OperationId> {
        match self.val {
            Val::Obj { id, .. } => Some(id),
            Val::Slice { backing, .. } => Some(backing),
            _ => None,
        }
    }
    fn with(&self, events: &BTreeSet<Event>) -> Self {
        let mut next = self.clone();
        next.events.extend(events.iter().copied());
        next
    }
}

type Refs = BTreeSet<Ref>;

#[derive(Clone, Copy)]
struct ReadSite<'a> {
    span: Span,
    path: &'a Option<RawPath>,
    ambiguous: bool,
    slot: Option<&'a Slot>,
}

#[derive(Debug, Clone, Default)]
struct Args {
    positional: Vec<Refs>,
    named: BTreeMap<String, Refs>,
}

impl Args {
    fn positional(positional: Vec<Refs>) -> Self {
        Self {
            positional,
            named: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Obj {
    container: Option<Container>,
    type_name: Option<String>,
    fields: BTreeMap<String, Refs>,
    items: BTreeMap<u64, Refs>,
    rest: Refs,
    length: Option<u64>,
    value_semantics: bool,
    opaque: bool,
}

#[derive(Debug, Clone, Default)]
struct State {
    env: BTreeMap<(Span, usize), Refs>,
    heap: BTreeMap<OperationId, Obj>,
    writes: BTreeMap<(OperationId, Slot), BTreeSet<usize>>,
    escaped: BTreeMap<OperationId, usize>,
    terminated: bool,
}

fn union(into: &mut Refs, from: &Refs) {
    for item in from {
        if into.len() >= MAX_REFS {
            break;
        }
        into.insert(item.clone());
    }
}

fn join(a: State, b: State) -> State {
    if a.terminated && !b.terminated {
        return b;
    }
    if b.terminated && !a.terminated {
        return a;
    }
    let mut out = a;
    for (key, refs) in b.env {
        union(out.env.entry(key).or_default(), &refs);
    }
    for (id, obj) in b.heap {
        match out.heap.get_mut(&id) {
            None => {
                out.heap.insert(id, obj);
            }
            Some(existing) => {
                for (name, refs) in obj.fields {
                    union(existing.fields.entry(name).or_default(), &refs);
                }
                for (index, refs) in obj.items {
                    union(existing.items.entry(index).or_default(), &refs);
                }
                union(&mut existing.rest, &obj.rest);
                if existing.length != obj.length {
                    existing.length = None;
                }
                existing.opaque |= obj.opaque;
            }
        }
    }
    for (key, writes) in b.writes {
        out.writes.entry(key).or_default().extend(writes);
    }
    out.escaped.extend(b.escaped);
    out
}

struct Closure {
    func: Rc<Func>,
    name: Option<String>,
    context: usize,
    defaults: Vec<Option<(Refs, OperationId)>>,
    invoked: bool,
}

struct Frame {
    context: usize,
    function: Span,
    returns: Vec<(State, Refs)>,
}

pub struct Interp<'g> {
    graph: &'g mut StateGraph,
    uri: String,
    source: String,
    lang: Lang,
    state: State,
    closures: Vec<Closure>,
    methods: BTreeMap<(String, String), usize>,
    frames: Vec<Frame>,
    lexical: BTreeMap<usize, usize>,
    loop_depth: usize,
    try_depth: usize,
    guard_depth: usize,
    throws: Vec<Vec<State>>,
    steps: usize,
    exhausted: bool,
}

impl<'g> Interp<'g> {
    pub fn new(graph: &'g mut StateGraph, uri: &str, source: &str, lang: Lang) -> Self {
        Self {
            graph,
            uri: uri.into(),
            source: source.into(),
            lang,
            state: State::default(),
            closures: vec![],
            methods: BTreeMap::new(),
            frames: vec![],
            lexical: BTreeMap::new(),
            loop_depth: 0,
            try_depth: 0,
            guard_depth: 0,
            throws: vec![],
            steps: 0,
            exhausted: false,
        }
    }

    pub fn run(mut self, program: &[Stmt]) {
        self.block(program);
        let finished = self.state.clone();
        for _ in 0..MAX_STANDALONE_PASSES {
            let pending = |c: &Closure| !c.invoked;
            let entry = self
                .closures
                .iter()
                .position(|c| pending(c) && c.name.as_deref() == Some(ENTRY_POINT));
            let Some(index) = entry.or_else(|| self.closures.iter().position(pending)) else {
                break;
            };
            self.closures[index].invoked = true;
            let func = self.closures[index].func.clone();
            if func.opaque.is_some() {
                continue;
            }
            self.state = finished.clone();
            let call = self.op(OperationKind::Call, func.span, None);
            let mut args = vec![];
            for param in &func.params {
                args.push(self.parameter_object(param));
            }
            let receiver = func.receiver.as_ref().map(|p| self.parameter_object(p));
            self.invoke(index, receiver, Args::positional(args), call, false);
        }
        if self.exhausted {
            self.limit(LIMIT_BUDGET);
        }
    }

    fn parameter_object(&mut self, param: &Param) -> Refs {
        let id = self.alloc(param.span, None, false);
        Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
    }

    fn limit(&mut self, text: &str) {
        if !self.graph.limitations.iter().any(|l| l == text) {
            self.graph.limitations.push(text.into());
        }
    }

    fn context(&self) -> usize {
        self.frames.last().map_or(0, |f| f.context)
    }

    fn chain(&self) -> Vec<usize> {
        let mut out = vec![self.context()];
        while let Some(parent) = self.lexical.get(out.last().unwrap()) {
            if out.contains(parent) {
                break;
            }
            out.push(*parent);
        }
        if !out.contains(&0) {
            out.push(0);
        }
        out
    }

    fn descends(&self, mut context: usize, ancestor: usize) -> bool {
        loop {
            if context == ancestor {
                return true;
            }
            if context == 0 {
                return false;
            }
            match self.graph.heap.contexts[context - 1].parent {
                Some(parent) => context = parent,
                None => context = 0,
            }
        }
    }

    fn op(&mut self, kind: OperationKind, span: Span, binding: Option<Span>) -> OperationId {
        let id = self.graph.operations.len();
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
                uri: self.uri.clone(),
                range: span,
            },
            binding,
            guards: vec![],
        });
        self.graph.heap.ops.insert(id);
        let context = self.context();
        self.graph.heap.op_context.insert(id, context);
        id
    }

    fn edge(
        &mut self,
        from: OperationId,
        to: OperationId,
        relationship: Relationship,
        certainty: Option<Certainty>,
    ) {
        if !self
            .graph
            .edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.relationship == relationship)
        {
            self.graph.edges.push(Edge {
                from,
                to,
                relationship,
                certainty,
            });
        }
    }

    fn boundary(&mut self, span: Span, kind: &'static str, reason: &'static str) -> usize {
        let op = self.op(OperationKind::Call, span, None);
        self.graph
            .heap
            .boundaries
            .push(RawBoundary { op, kind, reason });
        self.graph.heap.boundaries.len() - 1
    }

    fn alloc(
        &mut self,
        span: Span,
        container: Option<Container>,
        value_semantics: bool,
    ) -> OperationId {
        let id = self.op(OperationKind::Allocation, span, None);
        self.graph.heap.allocation_ranges.insert(id, span);
        if let Some(container) = container {
            self.graph.heap.containers.insert(id, container);
        }
        self.state.heap.insert(
            id,
            Obj {
                container,
                value_semantics,
                ..Obj::default()
            },
        );
        id
    }

    fn share(
        &mut self,
        op: OperationId,
        allocation: OperationId,
        kind: Sharing,
        qualifier: Option<u64>,
    ) {
        self.graph.heap.sharing.insert(RawSharing {
            op,
            allocation,
            kind,
            qualifier,
        });
    }

    fn step(&mut self) -> bool {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            self.exhausted = true;
        }
        !self.exhausted
    }

    fn certainty(&self, ambiguous: bool) -> Certainty {
        if ambiguous || self.loop_depth > 0 || self.try_depth > 0 {
            Certainty::Possible
        } else {
            Certainty::Definite
        }
    }

    fn path(&self, expr: &Expr) -> Option<RawPath> {
        match expr {
            Expr::Var {
                key: Key::Binding(decl),
                ..
            } => Some(RawPath {
                root: Root::Binding(*decl),
                segments: vec![],
            }),
            Expr::Field { base, name, .. } => self.path(base).map(|mut p| {
                p.segments.push(Segment::Field(name.clone()));
                p
            }),
            Expr::Index { base, index, .. } => self.path(base).map(|mut p| {
                p.segments.push(match index {
                    Index::Constant(i) => Segment::Element(Some(*i)),
                    Index::Name(n) => Segment::Field(n.clone()),
                    Index::Dynamic(_) => Segment::Element(None),
                });
                p
            }),
            _ => None,
        }
    }

    fn record_path(
        &mut self,
        op: OperationId,
        path: Option<RawPath>,
        fallback: Option<OperationId>,
    ) {
        let path = path.or_else(|| {
            fallback.map(|id| RawPath {
                root: Root::Allocation(id),
                segments: vec![],
            })
        });
        if let Some(path) = path {
            self.graph.heap.paths.insert(op, path);
        }
    }

    fn lookup(&self, decl: Span) -> Option<(usize, &Refs)> {
        self.chain()
            .into_iter()
            .find_map(|c| self.state.env.get(&(decl, c)).map(|r| (c, r)))
    }

    fn materialize(&mut self, refs: Refs, span: Span) -> Refs {
        if self.lang != Lang::Go {
            return refs;
        }
        let mut out = Refs::new();
        for r in refs {
            let copy = match r.val {
                Val::Obj { id, pointer: false } => {
                    self.state.heap.get(&id).is_some_and(|o| o.value_semantics)
                }
                _ => false,
            };
            if !copy {
                out.insert(r);
                continue;
            }
            let Val::Obj { id, .. } = r.val else {
                continue;
            };
            let source = self.state.heap[&id].clone();
            let copy_id = self.alloc(span, source.container, true);
            let mut fields = BTreeMap::new();
            for (name, refs) in &source.fields {
                fields.insert(name.clone(), self.copied(refs, copy_id, 0));
            }
            let mut items = BTreeMap::new();
            for (index, refs) in &source.items {
                items.insert(*index, self.copied(refs, copy_id, 0));
            }
            let rest = self.copied(&source.rest, copy_id, 0);
            let target = self.state.heap.get_mut(&copy_id).unwrap();
            target.fields = fields;
            target.items = items;
            target.rest = rest;
            target.type_name = source.type_name;
            target.length = source.length;
            target.opaque = source.opaque;
            out.insert(Ref {
                val: Val::Obj {
                    id: copy_id,
                    pointer: false,
                },
                events: r.events,
            });
        }
        out
    }

    fn copied(&mut self, refs: &Refs, op: OperationId, qualifier: u64) -> Refs {
        let mut out = Refs::new();
        for r in refs {
            match r.object() {
                Some(object) => {
                    self.share(op, object, Sharing::ShallowCopy, None);
                    let mut next = r.clone();
                    next.events.insert(Event {
                        op,
                        kind: Sharing::ShallowCopy,
                        qualifier,
                    });
                    out.insert(next);
                }
                None => {
                    out.insert(r.clone());
                }
            }
        }
        out
    }

    fn escape(&mut self, refs: &Refs, boundary: usize) {
        for object in self.reachable(refs) {
            self.state.escaped.entry(object).or_insert(boundary);
        }
    }

    fn reachable(&self, refs: &Refs) -> BTreeSet<OperationId> {
        let mut seen = BTreeSet::new();
        let mut work: VecDeque<(OperationId, usize)> = refs
            .iter()
            .filter_map(|r| r.object())
            .map(|o| (o, 0))
            .collect();
        while let Some((object, depth)) = work.pop_front() {
            if !seen.insert(object) || depth >= MAX_OBSERVE_DEPTH {
                continue;
            }
            if let Some(obj) = self.state.heap.get(&object) {
                for refs in obj
                    .fields
                    .values()
                    .chain(obj.items.values())
                    .chain([&obj.rest])
                {
                    for r in refs {
                        if let Some(child) = r.object() {
                            work.push_back((child, depth + 1));
                        }
                    }
                }
            }
        }
        seen
    }

    fn require(&mut self, refs: &Refs, kind: RequirementKind, declaration: Span) {
        for object in self.reachable(refs) {
            let entry = self.graph.heap.requirements.entry(object).or_default();
            if !entry.iter().any(|r| r.declaration == declaration) {
                entry.push(Requirement { kind, declaration });
            }
        }
    }

    fn slot_writes(&self, object: OperationId, slot: &Slot) -> BTreeSet<usize> {
        let mut out = BTreeSet::new();
        for ((o, s), writes) in &self.state.writes {
            if *o != object {
                continue;
            }
            let relevant = match (slot, s) {
                (Slot::Field(a), Slot::Field(b)) => a == b,
                (Slot::Field(_), Slot::AnyField) => true,
                (Slot::Element(a), Slot::Element(b)) => a == b,
                (Slot::Element(_), Slot::AnyElement) => true,
                (Slot::AnyElement, Slot::Element(_) | Slot::AnyElement) => true,
                (Slot::AnyField, Slot::Field(_) | Slot::AnyField) => true,
                _ => false,
            };
            if relevant {
                out.extend(writes.iter().copied());
            }
        }
        out
    }

    fn observe_object(
        &mut self,
        object: OperationId,
        writes: BTreeSet<usize>,
        read: &mut Option<OperationId>,
        site: &ReadSite<'_>,
        events: &BTreeSet<Event>,
        segments: Vec<Segment>,
    ) {
        let ReadSite {
            span,
            path,
            ambiguous,
            slot: read_slot,
        } = *site;
        if self.guard_depth > 0 && self.context() > 0 {
            self.graph.heap.guarded.insert(object);
        }
        if writes.is_empty() {
            return;
        }
        let read = *read.get_or_insert_with(|| {
            let op = self.op(OperationKind::Read, span, None);
            self.record_path(op, path.clone(), Some(object));
            op
        });
        self.edge(object, read, Relationship::PointsTo, None);
        let boundary = self.state.escaped.get(&object).copied();
        for write in writes {
            let fact = &self.graph.heap.writes[write];
            let imprecise = matches!(read_slot, Some(Slot::Field(_) | Slot::Element(_)))
                && matches!(fact.slot, Slot::AnyField | Slot::AnyElement);
            let certainty = if ambiguous || imprecise || fact.certainty == Certainty::Possible {
                Certainty::Possible
            } else {
                self.certainty(false)
            };
            let write_op = fact.op;
            let boundary = boundary.or(fact.boundary);
            self.edge(
                write_op,
                read,
                Relationship::HeapDependency,
                Some(certainty),
            );
            self.graph.heap.observations.push(Observation {
                write,
                read,
                events: events.clone(),
                certainty,
                segments: segments.clone(),
                boundary,
            });
        }
    }

    fn observe_deep(&mut self, refs: &Refs, span: Span, path: Option<RawPath>) {
        let mut read = None;
        let mut seen = BTreeSet::new();
        let mut work: VecDeque<(Ref, Vec<Segment>, usize)> =
            refs.iter().cloned().map(|r| (r, vec![], 0)).collect();
        while let Some((r, segments, depth)) = work.pop_front() {
            let Some(object) = r.object() else { continue };
            if !seen.insert((object, r.events.clone())) {
                continue;
            }
            let all = self.slot_writes(object, &Slot::AnyField);
            let mut writes = self.slot_writes(object, &Slot::AnyElement);
            writes.extend(all);
            if let Val::Slice {
                offset: Some(offset),
                length: Some(length),
                ..
            } = r.val
            {
                writes.retain(|w| match &self.graph.heap.writes[*w].slot {
                    Slot::Element(i) => *i >= offset && *i < offset + length,
                    _ => true,
                });
            }
            let site = ReadSite {
                span,
                path: &path,
                ambiguous: false,
                slot: None,
            };
            self.observe_object(
                object,
                writes,
                &mut read,
                &site,
                &r.events,
                segments.clone(),
            );
            if depth >= MAX_OBSERVE_DEPTH {
                continue;
            }
            let Some(obj) = self.state.heap.get(&object).cloned() else {
                continue;
            };
            let (low, high) = match r.val {
                Val::Slice {
                    offset: Some(offset),
                    length: Some(length),
                    ..
                } => (offset, offset + length),
                _ => (0, u64::MAX),
            };
            for (name, children) in &obj.fields {
                for child in children {
                    let mut s = segments.clone();
                    s.push(Segment::Field(name.clone()));
                    work.push_back((child.with(&r.events), s, depth + 1));
                }
            }
            for (index, children) in &obj.items {
                if *index < low || *index >= high {
                    continue;
                }
                for child in children {
                    let mut s = segments.clone();
                    s.push(Segment::Element(Some(*index - low)));
                    work.push_back((child.with(&r.events), s, depth + 1));
                }
            }
            for child in &obj.rest {
                let mut s = segments.clone();
                s.push(Segment::Element(None));
                work.push_back((child.with(&r.events), s, depth + 1));
            }
        }
    }

    fn element(&self, object: OperationId, index: Option<u64>) -> Refs {
        let Some(obj) = self.state.heap.get(&object) else {
            return Refs::new();
        };
        match index {
            Some(i) => match obj.items.get(&i) {
                Some(refs) => refs.clone(),
                None => obj
                    .rest
                    .iter()
                    .map(|r| {
                        let mut r = r.clone();
                        r.events = r
                            .events
                            .into_iter()
                            .map(|mut e| {
                                if e.qualifier == ANY_ELEMENT && e.kind == Sharing::RepeatedElement
                                {
                                    e.qualifier = i;
                                }
                                e
                            })
                            .collect();
                        r
                    })
                    .collect(),
            },
            None => {
                let mut out = obj.rest.clone();
                for refs in obj.items.values() {
                    union(&mut out, refs);
                }
                out.into_iter()
                    .map(|mut r| {
                        r.events = r
                            .events
                            .into_iter()
                            .map(|mut e| {
                                if e.kind == Sharing::RepeatedElement {
                                    e.qualifier = ANY_ELEMENT;
                                }
                                e
                            })
                            .collect();
                        r
                    })
                    .collect()
            }
        }
    }

    fn read_member(&mut self, base: &Refs, slot: Slot, span: Span, path: Option<RawPath>) -> Refs {
        let mut out = Refs::new();
        let mut read = None;
        let ambiguous = base
            .iter()
            .filter_map(|r| r.object())
            .collect::<BTreeSet<_>>()
            .len()
            > 1;
        for r in base {
            match &r.val {
                Val::Unknown(b) => {
                    out.insert(Ref::plain(Val::Unknown(*b)));
                }
                Val::Obj { id, .. } => {
                    let id = *id;
                    self.graph.heap.reads.push(super::ReadFact {
                        range: span,
                        object: id,
                        property: match &slot {
                            Slot::Field(name) => Some(name.clone()),
                            _ => None,
                        },
                        context: self.context(),
                    });
                    let opaque = self.state.heap.get(&id).is_some_and(|o| o.opaque);
                    let writes = self.slot_writes(id, &slot);
                    let site = ReadSite {
                        span,
                        path: &path,
                        ambiguous,
                        slot: Some(&slot),
                    };
                    self.observe_object(id, writes, &mut read, &site, &r.events, vec![]);
                    let children = match &slot {
                        Slot::Field(name) => self
                            .state
                            .heap
                            .get(&id)
                            .and_then(|o| o.fields.get(name))
                            .cloned()
                            .unwrap_or_default(),
                        Slot::Element(i) => self.element(id, Some(*i)),
                        Slot::AnyElement => self.element(id, None),
                        Slot::AnyField => self
                            .state
                            .heap
                            .get(&id)
                            .map(|o| {
                                let mut all = Refs::new();
                                for refs in o.fields.values() {
                                    union(&mut all, refs);
                                }
                                all
                            })
                            .unwrap_or_default(),
                    };
                    if opaque && children.is_empty() {
                        let b = self.boundary(span, "opaque-object", "Property read from an object with accessors or computed members is not modeled.");
                        out.insert(Ref::plain(Val::Unknown(b)));
                    }
                    for child in children {
                        out.insert(child.with(&r.events));
                    }
                }
                Val::Slice {
                    backing, offset, ..
                } => {
                    let index = match (&slot, offset) {
                        (Slot::Element(i), Some(o)) => Some(Slot::Element(i + o)),
                        _ => None,
                    };
                    let slot = index.unwrap_or(Slot::AnyElement);
                    let writes = self.slot_writes(*backing, &slot);
                    let site = ReadSite {
                        span,
                        path: &path,
                        ambiguous,
                        slot: Some(&slot),
                    };
                    self.observe_object(*backing, writes, &mut read, &site, &r.events, vec![]);
                    let index = match slot {
                        Slot::Element(i) => Some(i),
                        _ => None,
                    };
                    for child in self.element(*backing, index) {
                        out.insert(child.with(&r.events));
                    }
                }
                _ => {}
            }
        }
        out
    }

    fn write_member(
        &mut self,
        base: &Refs,
        slot: Slot,
        value: Refs,
        span: Span,
        path: Option<RawPath>,
        append: bool,
    ) {
        let targets: Vec<Ref> = base
            .iter()
            .filter(|r| r.object().is_some())
            .cloned()
            .collect();
        if targets.is_empty() {
            return;
        }
        let op = self.op(OperationKind::Write, span, None);
        self.record_path(op, path, targets[0].object());
        let ambiguous = targets
            .iter()
            .filter_map(|r| r.object())
            .collect::<BTreeSet<_>>()
            .len()
            > 1;
        for r in &targets {
            let (object, slot) = match (&r.val, &slot) {
                (
                    Val::Slice {
                        backing,
                        offset: Some(o),
                        ..
                    },
                    Slot::Element(i),
                ) => (*backing, Slot::Element(i + o)),
                (Val::Slice { backing, .. }, Slot::Element(_)) => (*backing, Slot::AnyElement),
                (Val::Slice { backing, .. }, s) => (*backing, s.clone()),
                (_, s) => (r.object().unwrap(), s.clone()),
            };
            let weak = ambiguous || matches!(slot, Slot::AnyElement | Slot::AnyField);
            let certainty = self.certainty(ambiguous);
            self.edge(object, op, Relationship::PointsTo, None);
            let boundary = self.state.escaped.get(&object).copied();
            self.graph.heap.writes.push(WriteFact {
                op,
                object,
                slot: slot.clone(),
                events: r.events.clone(),
                certainty,
                append,
                boundary,
                context: self.context(),
            });
            let index = self.graph.heap.writes.len() - 1;
            let strong = !weak && self.loop_depth == 0;
            let entry = self.state.writes.entry((object, slot.clone())).or_default();
            if strong {
                entry.clear();
            }
            entry.insert(index);
            let Some(obj) = self.state.heap.get_mut(&object) else {
                continue;
            };
            match &slot {
                Slot::Field(name) => {
                    if strong {
                        obj.fields.insert(name.clone(), value.clone());
                    } else {
                        union(obj.fields.entry(name.clone()).or_default(), &value);
                    }
                }
                Slot::Element(i) => {
                    if strong {
                        obj.items.insert(*i, value.clone());
                    } else {
                        union(obj.items.entry(*i).or_default(), &value);
                    }
                }
                Slot::AnyElement => union(&mut obj.rest, &value),
                Slot::AnyField => {}
            }
        }
    }

    fn eval(&mut self, expr: &Expr, mode: Mode) -> Refs {
        if !self.step() {
            return Refs::new();
        }
        match expr {
            Expr::Var { key, span, .. } => {
                let Key::Binding(decl) = key else {
                    return Refs::new();
                };
                let refs = self
                    .lookup(*decl)
                    .map(|(_, r)| r.clone())
                    .unwrap_or_default();
                if mode == Mode::Use && refs.iter().any(|r| r.object().is_some()) {
                    self.observe_deep(&refs, *span, self.path(expr));
                }
                refs
            }
            Expr::Field { base, name, span } => {
                if matches!(
                    **base,
                    Expr::Var {
                        key: Key::Builtin,
                        ..
                    }
                ) {
                    return Refs::new();
                }
                let b = self.eval(base, Mode::Base);
                let refs = self.read_member(&b, Slot::Field(name.clone()), *span, self.path(expr));
                if mode == Mode::Use {
                    self.observe_deep(&refs, *span, self.path(expr));
                }
                refs
            }
            Expr::Index { base, index, span } => {
                let b = self.eval(base, Mode::Base);
                let slot = self.slot(&b, index, *span);
                let refs = self.read_member(&b, slot, *span, self.path(expr));
                if mode == Mode::Use {
                    self.observe_deep(&refs, *span, self.path(expr));
                }
                refs
            }
            Expr::Call { callee, args, span } => {
                let refs = self.call(callee, args, *span);
                if mode == Mode::Use {
                    self.observe_deep(&refs, *span, None);
                }
                refs
            }
            Expr::Literal {
                container,
                type_name,
                entries,
                elements,
                span,
                value_semantics,
            } => self.literal(
                *container,
                type_name,
                entries,
                elements,
                *span,
                *value_semantics,
            ),
            Expr::NewArray { length, span } => {
                let length = length.as_ref().and_then(|l| self.integer(l));
                let id = self.alloc(*span, Some(Container::Array), false);
                self.state.heap.get_mut(&id).unwrap().length = length;
                Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
            }
            Expr::Repeat {
                element,
                count,
                span,
            } => {
                let source = self.eval(element, Mode::Alias);
                let count = self.integer(count);
                let id = self.alloc(*span, Some(Container::Array), false);
                let mut items = BTreeMap::new();
                let mut rest = Refs::new();
                for r in &source {
                    let Some(object) = r.object() else { continue };
                    let Some(obj) = self.state.heap.get(&object).cloned() else {
                        continue;
                    };
                    let width = obj.items.len() as u64;
                    let repeated = |s: &mut Self, refs: &Refs, qualifier: u64| -> Refs {
                        refs.iter()
                            .map(|child| {
                                if let Some(o) = child.object() {
                                    s.share(id, o, Sharing::RepeatedElement, None);
                                }
                                let mut c = child.clone();
                                c.events.insert(Event {
                                    op: id,
                                    kind: Sharing::RepeatedElement,
                                    qualifier,
                                });
                                c
                            })
                            .collect()
                    };
                    match count.filter(|c| c * width <= MAX_UNROLL * MAX_UNROLL) {
                        Some(count) => {
                            for k in 0..count {
                                for (j, refs) in &obj.items {
                                    let position = k * width + j;
                                    let refs = repeated(self, refs, position);
                                    items.insert(position, refs);
                                }
                            }
                        }
                        None => {
                            for refs in obj.items.values() {
                                let refs = repeated(self, refs, ANY_ELEMENT);
                                union(&mut rest, &refs);
                            }
                        }
                    }
                }
                let obj = self.state.heap.get_mut(&id).unwrap();
                obj.length = count.and_then(|_| (!items.is_empty()).then_some(items.len() as u64));
                obj.items = items;
                obj.rest = rest;
                Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
            }
            Expr::Comprehension {
                element,
                count,
                span,
            } => {
                let id = self.alloc(*span, Some(Container::Array), false);
                let unroll = count.unwrap_or(UNKNOWN_COUNT_UNROLL).min(MAX_UNROLL);
                for i in 0..unroll {
                    let refs = self.eval(element, Mode::Alias);
                    self.state.heap.get_mut(&id).unwrap().items.insert(i, refs);
                }
                self.state.heap.get_mut(&id).unwrap().length = *count;
                Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
            }
            Expr::Slice {
                base,
                low,
                high,
                max,
                span,
            } => {
                let refs = self.eval(base, Mode::Alias);
                let low = match low {
                    Some(l) => self.integer(l),
                    None => Some(0),
                };
                let high = high.as_ref().map(|h| self.integer(h));
                let max = max.as_ref().map(|m| self.integer(m));
                let op = self.op(OperationKind::Read, *span, None);
                self.record_path(op, self.path(base), None);
                let mut out = Refs::new();
                for r in refs {
                    let (backing, offset, length, capacity) = match r.val {
                        Val::Slice {
                            backing,
                            offset,
                            length,
                            capacity,
                        } => (backing, offset, length, capacity),
                        Val::Obj { id, .. } => {
                            let length = self.state.heap.get(&id).and_then(|o| o.length);
                            (id, Some(0), length, length)
                        }
                        _ => {
                            out.insert(r);
                            continue;
                        }
                    };
                    let start = offset.zip(low).map(|(o, l)| o + l);
                    let end = match high {
                        Some(h) => h,
                        None => length,
                    };
                    let new_length = end.zip(low).map(|(e, l)| e.saturating_sub(l));
                    let new_capacity = match max {
                        Some(m) => m.zip(low).map(|(m, l)| m.saturating_sub(l)),
                        None => capacity.zip(low).map(|(c, l)| c.saturating_sub(l)),
                    };
                    self.share(op, backing, Sharing::Reslice, None);
                    let mut events = r.events.clone();
                    events.insert(Event {
                        op,
                        kind: Sharing::Reslice,
                        qualifier: 0,
                    });
                    out.insert(Ref {
                        val: Val::Slice {
                            backing,
                            offset: start,
                            length: new_length,
                            capacity: new_capacity,
                        },
                        events,
                    });
                }
                if mode == Mode::Use {
                    self.observe_deep(&out, *span, self.path(base));
                }
                out
            }
            Expr::AddressOf { operand, .. } => {
                let refs = match &**operand {
                    Expr::Literal { .. } => self.eval(operand, Mode::Alias),
                    other => self.eval(other, Mode::Base),
                };
                refs.into_iter()
                    .map(|mut r| {
                        if let Val::Obj { pointer, .. } = &mut r.val {
                            *pointer = true;
                        }
                        r
                    })
                    .collect()
            }
            Expr::Function(func) => self.closure(func.clone()),
            Expr::Integer(value, _) => Refs::from([Ref::plain(Val::Int(*value))]),
            Expr::Null(_) => Refs::from([Ref::plain(Val::Null)]),
            Expr::Choice(options, _) => {
                let mut out = Refs::new();
                for option in options {
                    let refs = self.eval(option, mode);
                    union(&mut out, &refs);
                }
                out
            }
            Expr::Sequence(items, _) => {
                let mut last = Refs::new();
                for (i, item) in items.iter().enumerate() {
                    last = self.eval(
                        item,
                        if i + 1 == items.len() {
                            mode
                        } else {
                            Mode::Use
                        },
                    );
                }
                last
            }
            Expr::Assign(stmt, _) => self.assign_stmt(stmt),
            Expr::Named(_, inner, _) => self.eval(inner, mode),
            Expr::Variadic(inner, _) => {
                let refs = self.eval(inner, mode);
                self.element_union(&refs)
            }
            Expr::Scalar(inputs, _) => {
                for input in inputs {
                    self.eval(input, Mode::Use);
                }
                Refs::new()
            }
            Expr::Unknown {
                reason,
                inputs,
                span,
            } => {
                let b = self.boundary(*span, "unmodeled-expression", reason);
                for input in inputs {
                    let refs = self.eval(input, Mode::Use);
                    self.escape(&refs, b);
                }
                Refs::from([Ref::plain(Val::Unknown(b))])
            }
        }
    }

    fn integer(&mut self, expr: &Expr) -> Option<u64> {
        let refs = self.eval(expr, Mode::Use);
        match refs.iter().next().map(|r| &r.val) {
            Some(Val::Int(v)) if refs.len() == 1 && *v >= 0 => Some(*v as u64),
            _ => None,
        }
    }

    fn slot(&mut self, base: &Refs, index: &Index, span: Span) -> Slot {
        match index {
            Index::Constant(i) => {
                let keyed = base.iter().any(|r| {
                    r.object().is_some_and(|o| {
                        self.state.heap.get(&o).is_some_and(|o| {
                            matches!(o.container, Some(Container::Map | Container::Object))
                        })
                    }) && !matches!(r.val, Val::Slice { .. })
                });
                if keyed {
                    Slot::Field(i.to_string())
                } else {
                    Slot::Element(*i)
                }
            }
            Index::Name(name) => Slot::Field(name.clone()),
            Index::Dynamic(expr) => {
                let value = self.integer(expr);
                let keyed = base.iter().any(|r| {
                    r.object().is_some_and(|o| {
                        self.state.heap.get(&o).is_some_and(|o| {
                            matches!(o.container, Some(Container::Map | Container::Object) | None)
                        })
                    }) && !matches!(r.val, Val::Slice { .. })
                });
                match (value, keyed) {
                    (Some(i), false) => Slot::Element(i),
                    (_, false) => Slot::AnyElement,
                    (_, true) => {
                        if self.lang == Lang::JavaScript {
                            self.boundary(
                                span,
                                "dynamic-key",
                                "Computed property keys are not resolved.",
                            );
                        }
                        Slot::AnyField
                    }
                }
            }
        }
    }

    fn literal(
        &mut self,
        container: Container,
        type_name: &Option<String>,
        entries: &[Entry],
        elements: &[Expr],
        span: Span,
        value_semantics: bool,
    ) -> Refs {
        let id = self.alloc(span, Some(container), value_semantics);
        self.state.heap.get_mut(&id).unwrap().type_name = type_name.clone();
        let mut position = 0u64;
        for entry in entries {
            match entry {
                Entry::Field(name, value) => {
                    let refs = self.eval(value, Mode::Alias);
                    let refs = self.materialize(refs, value.span());
                    self.state
                        .heap
                        .get_mut(&id)
                        .unwrap()
                        .fields
                        .insert(name.clone(), refs);
                }
                Entry::Spread(source) => {
                    let refs = self.eval(source, Mode::Alias);
                    let op = self.op(OperationKind::Read, source.span(), None);
                    self.record_path(op, self.path(source), None);
                    for r in &refs {
                        match &r.val {
                            Val::Unknown(_) => {
                                self.state.heap.get_mut(&id).unwrap().opaque = true;
                            }
                            _ => {
                                let Some(object) = r.object() else { continue };
                                let Some(src) = self.state.heap.get(&object).cloned() else {
                                    continue;
                                };
                                if src.opaque {
                                    self.state.heap.get_mut(&id).unwrap().opaque = true;
                                }
                                if container == Container::Array {
                                    for children in src.items.values() {
                                        let children: Refs =
                                            children.iter().map(|c| c.with(&r.events)).collect();
                                        let copied = self.copied(&children, op, 0);
                                        self.state
                                            .heap
                                            .get_mut(&id)
                                            .unwrap()
                                            .items
                                            .insert(position, copied);
                                        position += 1;
                                    }
                                    let rest: Refs =
                                        src.rest.iter().map(|c| c.with(&r.events)).collect();
                                    let copied = self.copied(&rest, op, 0);
                                    let target = self.state.heap.get_mut(&id).unwrap();
                                    union(&mut target.rest, &copied);
                                    if !src.rest.is_empty() {
                                        target.length = None;
                                    }
                                } else {
                                    for (name, children) in &src.fields {
                                        let children: Refs =
                                            children.iter().map(|c| c.with(&r.events)).collect();
                                        let copied = self.copied(&children, op, 0);
                                        self.state
                                            .heap
                                            .get_mut(&id)
                                            .unwrap()
                                            .fields
                                            .insert(name.clone(), copied);
                                    }
                                }
                            }
                        }
                    }
                }
                Entry::Element(value) => {
                    let refs = self.eval(value, Mode::Alias);
                    let refs = self.materialize(refs, value.span());
                    self.state
                        .heap
                        .get_mut(&id)
                        .unwrap()
                        .items
                        .insert(position, refs);
                    position += 1;
                }
                Entry::Opaque(reason, at) => {
                    self.boundary(*at, "opaque-member", reason);
                    self.state.heap.get_mut(&id).unwrap().opaque = true;
                }
            }
        }
        for element in elements {
            let refs = self.eval(element, Mode::Alias);
            let refs = self.materialize(refs, element.span());
            self.state
                .heap
                .get_mut(&id)
                .unwrap()
                .items
                .insert(position, refs);
            position += 1;
        }
        let obj = self.state.heap.get_mut(&id).unwrap();
        if container == Container::Array && obj.rest.is_empty() {
            obj.length = Some(position);
        }
        if self.lang == Lang::Go && container == Container::Array && !value_semantics {
            let length = obj.length;
            return Refs::from([Ref::plain(Val::Slice {
                backing: id,
                offset: Some(0),
                length,
                capacity: length,
            })]);
        }
        Refs::from([Ref::plain(Val::Obj {
            id,
            pointer: !value_semantics,
        })])
    }

    fn closure(&mut self, func: Rc<Func>) -> Refs {
        let mut defaults = vec![];
        if func.defaults_at_definition {
            for param in &func.params {
                defaults.push(match &param.default {
                    Some(expr) => {
                        let op = self.op(OperationKind::Read, expr.span(), None);
                        let refs = self.eval(expr, Mode::Alias);
                        Some((refs, op))
                    }
                    None => None,
                });
            }
        }
        self.closures.push(Closure {
            func,
            name: None,
            context: self.context(),
            defaults,
            invoked: false,
        });
        Refs::from([Ref::plain(Val::Func(self.closures.len() - 1))])
    }

    fn arguments(&mut self, args: &[Expr]) -> Args {
        let mut out = Args::default();
        for a in args {
            match a {
                Expr::Named(name, value, _) => {
                    let refs = self.eval(value, Mode::Alias);
                    out.named.insert(name.clone(), refs);
                }
                other => {
                    let refs = self.eval(other, Mode::Alias);
                    out.positional.push(refs);
                }
            }
        }
        out
    }

    fn observe_all(&mut self, args: &[Expr]) -> Vec<Refs> {
        args.iter().map(|a| self.eval(a, Mode::Use)).collect()
    }

    fn unknown_call(
        &mut self,
        args: &[Expr],
        receiver: Option<Refs>,
        span: Span,
        reason: &'static str,
    ) -> Refs {
        let b = self.boundary(span, "unknown-call", reason);
        for refs in self.observe_all(args) {
            self.escape(&refs, b);
        }
        if let Some(refs) = receiver {
            self.escape(&refs, b);
        }
        Refs::from([Ref::plain(Val::Unknown(b))])
    }

    fn call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> Refs {
        if let Expr::Var {
            key: Key::Builtin,
            name,
            ..
        } = callee
        {
            return self.builtin_function(name, args, span);
        }
        if let Expr::Field { base, name, .. } = callee {
            if let Expr::Var {
                key: Key::Builtin,
                name: namespace,
                ..
            } = &**base
            {
                return self.builtin_namespace(namespace, name, args, span);
            }
            let receiver = self.eval(base, Mode::Base);
            if let Some(refs) = self.method(&receiver, base, name, args, span) {
                return refs;
            }
            let mut closures = BTreeSet::new();
            if self.lang == Lang::Go {
                for r in &receiver {
                    if let Some(object) = r.object() {
                        if let Some(t) = self
                            .state
                            .heap
                            .get(&object)
                            .and_then(|o| o.type_name.clone())
                        {
                            if let Some(c) = self.methods.get(&(t, name.clone())) {
                                closures.insert(*c);
                            }
                        }
                    }
                }
            }
            if closures.len() == 1 {
                let closure = *closures.iter().next().unwrap();
                let call = self.op(OperationKind::Call, span, None);
                let args = self.arguments(args);
                return self.invoke(closure, Some(receiver), args, call, true);
            }
            let fields = self.read_member(
                &receiver,
                Slot::Field(name.clone()),
                span,
                self.path(callee),
            );
            let funcs: Vec<usize> = fields
                .iter()
                .filter_map(|r| match r.val {
                    Val::Func(f) => Some(f),
                    _ => None,
                })
                .collect();
            if funcs.len() == 1 && fields.len() == 1 {
                let call = self.op(OperationKind::Call, span, None);
                let args = self.arguments(args);
                return self.invoke(funcs[0], None, args, call, true);
            }
            return self.unknown_call(
                args,
                Some(receiver),
                span,
                "Method dispatch is not resolved; the receiver and arguments may be mutated.",
            );
        }
        let target = self.eval(callee, Mode::Base);
        let funcs: Vec<usize> = target
            .iter()
            .filter_map(|r| match r.val {
                Val::Func(f) => Some(f),
                _ => None,
            })
            .collect();
        if funcs.is_empty() || funcs.len() != target.len() {
            return self.unknown_call(
                args,
                None,
                span,
                "Callee is not a resolved local function; argument effects are unknown.",
            );
        }
        let call = self.op(OperationKind::Call, span, None);
        let args = self.arguments(args);
        let before = self.state.clone();
        let mut result = Refs::new();
        let mut states = vec![];
        for f in funcs {
            self.state = before.clone();
            let refs = self.invoke(f, None, args.clone(), call, true);
            union(&mut result, &refs);
            states.push(self.state.clone());
        }
        self.state = states.into_iter().reduce(join).unwrap_or(before);
        result
    }

    fn invoke(
        &mut self,
        closure: usize,
        receiver: Option<Refs>,
        args: Args,
        call: OperationId,
        apply_defaults: bool,
    ) -> Refs {
        let func = self.closures[closure].func.clone();
        if func.opaque.is_some()
            || self.frames.len() >= MAX_CALL_DEPTH
            || self.frames.iter().any(|f| f.function == func.span)
        {
            let b = self.boundary(
                self.graph.operations[call].location.range,
                "call-bound",
                "Recursive, deeply nested or opaque calls are not inlined.",
            );
            for refs in args
                .positional
                .iter()
                .chain(args.named.values())
                .chain(receiver.iter())
            {
                self.escape(refs, b);
            }
            return Refs::from([Ref::plain(Val::Unknown(b))]);
        }
        self.closures[closure].invoked = true;
        self.graph.heap.contexts.push(RawContext {
            parent: (self.context() != 0).then_some(self.context()),
            call,
        });
        let context = self.graph.heap.contexts.len();
        self.lexical.insert(context, self.closures[closure].context);
        self.frames.push(Frame {
            context,
            function: func.span,
            returns: vec![],
        });
        for (i, param) in func.params.iter().enumerate() {
            let refs = match args
                .positional
                .get(i)
                .or_else(|| args.named.get(&param.name))
            {
                Some(refs) => self.materialize(refs.clone(), param.span),
                None if apply_defaults => self.default(closure, i, param, context),
                None => Refs::new(),
            };
            if let Some((kind, at)) = param.requirement {
                self.require(&refs, kind, at);
            }
            self.state.env.insert((param.span, context), refs);
        }
        if let (Some(param), Some(refs)) = (&func.receiver, receiver) {
            let refs = if func.receiver_by_value {
                self.materialize(refs, param.span)
            } else {
                refs
            };
            self.state.env.insert((param.span, context), refs);
        }
        let mut fallthrough = Refs::new();
        if let Some(expr) = &func.expression_body {
            fallthrough = self.eval(expr, Mode::Alias);
            fallthrough = self.returned(fallthrough, expr, expr.span());
        } else {
            self.block(&func.body);
        }
        let frame = self.frames.pop().unwrap();
        let mut states = frame.returns;
        if !self.state.terminated {
            states.push((self.state.clone(), fallthrough));
        }
        let mut result = Refs::new();
        let mut joined: Option<State> = None;
        for (state, refs) in states {
            union(&mut result, &refs);
            joined = Some(match joined {
                None => state,
                Some(prior) => join(prior, state),
            });
        }
        match joined {
            Some(mut state) => {
                state.terminated = false;
                self.state = state;
            }
            None => self.state.terminated = true,
        }
        result
    }

    fn default(&mut self, closure: usize, index: usize, param: &Param, context: usize) -> Refs {
        if let Some(Some((refs, op))) = self.closures[closure].defaults.get(index).cloned() {
            return self.shared_default(refs, op, context);
        }
        let Some(expr) = &param.default else {
            return Refs::new();
        };
        let before = self.graph.operations.len();
        let refs = self.eval(expr, Mode::Alias);
        let preexisting = refs.iter().any(|r| r.object().is_some_and(|o| o < before));
        if !preexisting {
            return refs;
        }
        let op = self.op(OperationKind::Read, expr.span(), None);
        self.record_path(op, self.path(expr), None);
        let mut out = Refs::new();
        for r in refs {
            match r.object() {
                Some(o) if o < before => {
                    self.share(op, o, Sharing::SharedDefault, None);
                    let mut next = r.clone();
                    next.events.insert(Event {
                        op,
                        kind: Sharing::SharedDefault,
                        qualifier: context as u64,
                    });
                    out.insert(next);
                }
                _ => {
                    out.insert(r);
                }
            }
        }
        out
    }

    fn shared_default(&mut self, refs: Refs, op: OperationId, context: usize) -> Refs {
        refs.into_iter()
            .map(|r| match r.object() {
                Some(o) => {
                    self.share(op, o, Sharing::SharedDefault, None);
                    let mut next = r.clone();
                    next.events.insert(Event {
                        op,
                        kind: Sharing::SharedDefault,
                        qualifier: context as u64,
                    });
                    next
                }
                None => r,
            })
            .collect()
    }

    fn returned(&mut self, refs: Refs, expr: &Expr, span: Span) -> Refs {
        let context = self.context();
        if context == 0 {
            return refs;
        }
        let internal = matches!(expr, Expr::Field { .. } | Expr::Index { .. });
        let kind = if internal {
            Sharing::ReturnedInternal
        } else {
            Sharing::ReturnedAlias
        };
        let mut op = None;
        let mut out = Refs::new();
        for r in refs {
            let Some(object) = r.object() else {
                out.insert(r);
                continue;
            };
            let allocated_here = self.descends(self.graph.heap.context_of(object), context);
            if allocated_here {
                out.insert(r);
                continue;
            }
            let op = *op.get_or_insert_with(|| self.op(OperationKind::Return, span, None));
            self.record_path(op, self.path(expr), Some(object));
            self.share(op, object, kind, None);
            let mut next = r.clone();
            next.events.insert(Event {
                op,
                kind,
                qualifier: context as u64,
            });
            out.insert(next);
        }
        out
    }

    fn method(
        &mut self,
        receiver: &Refs,
        base: &Expr,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> Option<Refs> {
        if receiver.iter().any(|r| matches!(r.val, Val::Unknown(_))) {
            return None;
        }
        let containers: BTreeSet<Option<Container>> = receiver
            .iter()
            .filter_map(|r| match r.val {
                Val::Obj { id, .. } => Some(self.state.heap.get(&id).and_then(|o| o.container)),
                Val::Slice { .. } => Some(Some(Container::Array)),
                _ => None,
            })
            .collect();
        if containers.len() != 1 {
            return None;
        }
        let container = (*containers.iter().next().unwrap())?;
        let path = self.path(base);
        let element_path = path.clone().map(|mut p| {
            p.segments.push(Segment::Element(None));
            p
        });
        match (self.lang, container, name) {
            (Lang::JavaScript, Container::Array, "fill") => {
                let value = args
                    .first()
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                for a in args.iter().skip(1) {
                    self.eval(a, Mode::Use);
                }
                let call = self.op(OperationKind::Call, span, None);
                self.record_path(call, path.clone(), receiver.iter().find_map(|r| r.object()));
                for r in receiver {
                    let Some(object) = r.object() else { continue };
                    for v in &value {
                        if let Some(o) = v.object() {
                            self.share(call, o, Sharing::RepeatedElement, None);
                        }
                    }
                    let tagged = |qualifier: u64| -> Refs {
                        value
                            .iter()
                            .map(|v| {
                                let mut v = v.clone();
                                if v.object().is_some() {
                                    v.events.insert(Event {
                                        op: call,
                                        kind: Sharing::RepeatedElement,
                                        qualifier,
                                    });
                                }
                                v
                            })
                            .collect()
                    };
                    let length = self.state.heap.get(&object).and_then(|o| o.length);
                    let obj = self.state.heap.get_mut(&object).unwrap();
                    match length.filter(|l| *l <= MAX_UNROLL * MAX_UNROLL) {
                        Some(length) => {
                            obj.items = (0..length).map(|i| (i, tagged(i))).collect();
                            obj.rest.clear();
                        }
                        None => {
                            obj.items.clear();
                            obj.rest = tagged(ANY_ELEMENT);
                        }
                    }
                }
                Some(receiver.clone())
            }
            (Lang::JavaScript, Container::Array, "push" | "unshift")
            | (Lang::Python, Container::Array, "append" | "extend" | "insert")
            | (Lang::Python, Container::Set, "add" | "update") => {
                let values: Vec<Refs> = args.iter().map(|a| self.eval(a, Mode::Alias)).collect();
                let mut value = Refs::new();
                for v in &values {
                    union(&mut value, v);
                }
                if self.lang == Lang::Python && name == "extend" {
                    let mut elements = Refs::new();
                    for r in &value {
                        if let Some(o) = r.object() {
                            union(&mut elements, &self.element(o, None));
                        }
                    }
                    value = elements;
                }
                self.write_member(receiver, Slot::AnyElement, value, span, element_path, true);
                for r in receiver {
                    if let Val::Obj { id, .. } = r.val {
                        if let Some(o) = self.state.heap.get_mut(&id) {
                            o.length = None;
                        }
                    }
                }
                Some(Refs::new())
            }
            (
                Lang::JavaScript,
                Container::Array,
                "pop" | "shift" | "splice" | "sort" | "reverse" | "copyWithin",
            )
            | (Lang::Python, Container::Array, "pop" | "remove" | "clear" | "sort" | "reverse")
            | (Lang::Python, Container::Set, "discard" | "remove" | "clear" | "pop") => {
                self.observe_all(args);
                let removed = self.element_union(receiver);
                self.write_member(
                    receiver,
                    Slot::AnyElement,
                    Refs::new(),
                    span,
                    element_path,
                    false,
                );
                Some(
                    if matches!(name, "sort" | "reverse" | "copyWithin")
                        && self.lang == Lang::JavaScript
                    {
                        receiver.clone()
                    } else {
                        removed
                    },
                )
            }
            (
                Lang::Python,
                Container::Map,
                "update" | "setdefault" | "pop" | "popitem" | "clear",
            ) => {
                let values: Vec<Refs> = args.iter().map(|a| self.eval(a, Mode::Alias)).collect();
                let mut value = Refs::new();
                for v in values.iter().skip(1) {
                    union(&mut value, v);
                }
                let field_path = path.clone().map(|mut p| {
                    p.segments.push(Segment::Field("*".into()));
                    p
                });
                let current = self.read_member(receiver, Slot::AnyField, span, field_path.clone());
                self.write_member(receiver, Slot::AnyField, value, span, field_path, false);
                Some(current)
            }
            (Lang::Python, Container::Map, "get") => {
                let slot = match args.first() {
                    Some(Expr::Unknown { .. }) | None => Slot::AnyField,
                    Some(arg) => match self.literal_key(arg) {
                        Some(k) => Slot::Field(k),
                        None => {
                            self.eval(arg, Mode::Use);
                            Slot::AnyField
                        }
                    },
                };
                Some(self.read_member(receiver, slot, span, path))
            }
            (Lang::Python, Container::Array | Container::Map | Container::Set, "copy")
            | (Lang::JavaScript, Container::Array, "slice" | "concat") => {
                self.observe_all(args);
                Some(self.shallow_copy(receiver, span))
            }
            (
                Lang::JavaScript,
                Container::Array,
                "map" | "filter" | "forEach" | "some" | "every" | "find" | "findIndex" | "flatMap"
                | "reduce",
            ) => {
                let elements = self.element_union(receiver);
                let callbacks: Vec<Refs> = args.iter().map(|a| self.eval(a, Mode::Alias)).collect();
                let mut results = Refs::new();
                let funcs: Vec<usize> = callbacks
                    .first()
                    .map(|c| {
                        c.iter()
                            .filter_map(|r| match r.val {
                                Val::Func(f) => Some(f),
                                _ => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if funcs.len() == 1 {
                    let call = self.op(OperationKind::Call, span, None);
                    let refs = self.invoke(
                        funcs[0],
                        None,
                        Args::positional(vec![elements.clone()]),
                        call,
                        false,
                    );
                    union(&mut results, &refs);
                } else {
                    let b = self.boundary(
                        span,
                        "unknown-call",
                        "Collection callback is not a resolved local function.",
                    );
                    self.escape(receiver, b);
                }
                Some(match name {
                    "map" | "flatMap" => {
                        let id = self.alloc(span, Some(Container::Array), false);
                        self.state.heap.get_mut(&id).unwrap().rest = results;
                        Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
                    }
                    "filter" => self.shallow_copy(receiver, span),
                    "find" => elements,
                    _ => Refs::new(),
                })
            }
            (
                Lang::JavaScript,
                Container::Array,
                "join" | "indexOf" | "includes" | "toString" | "at" | "keys" | "values" | "entries",
            )
            | (Lang::Python, Container::Array | Container::Set, "index" | "count")
            | (Lang::Python, Container::Map, "keys" | "values" | "items") => {
                self.observe_deep(receiver, span, path);
                self.observe_all(args);
                Some(Refs::new())
            }
            _ => None,
        }
    }

    fn literal_key(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Scalar(inputs, span) if inputs.is_empty() => {
                let text = self.source.get(span.start..span.end)?;
                let trimmed = text.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                (trimmed.len() + 2 == text.len()).then(|| trimmed.to_string())
            }
            Expr::Integer(v, _) => Some(v.to_string()),
            _ => None,
        }
    }

    fn element_union(&mut self, receiver: &Refs) -> Refs {
        let mut out = Refs::new();
        for r in receiver {
            if let Some(object) = r.object() {
                let Some(obj) = self.state.heap.get(&object) else {
                    continue;
                };
                let mut elements = obj.rest.clone();
                for refs in obj.items.values() {
                    union(&mut elements, refs);
                }
                for e in elements {
                    out.insert(e.with(&r.events));
                }
            }
        }
        out
    }

    fn shallow_copy(&mut self, refs: &Refs, span: Span) -> Refs {
        let mut out = Refs::new();
        let op = self.op(OperationKind::Call, span, None);
        for r in refs {
            let Some(object) = r.object() else { continue };
            let Some(src) = self.state.heap.get(&object).cloned() else {
                continue;
            };
            let id = self.alloc(span, src.container, false);
            let mut fields = BTreeMap::new();
            for (name, children) in &src.fields {
                let children: Refs = children.iter().map(|c| c.with(&r.events)).collect();
                fields.insert(name.clone(), self.copied(&children, op, 0));
            }
            let (low, high) = match r.val {
                Val::Slice {
                    offset: Some(o),
                    length: Some(l),
                    ..
                } => (o, o + l),
                _ => (0, u64::MAX),
            };
            let mut items = BTreeMap::new();
            for (index, children) in &src.items {
                if *index < low || *index >= high {
                    continue;
                }
                let children: Refs = children.iter().map(|c| c.with(&r.events)).collect();
                items.insert(index - low, self.copied(&children, op, 0));
            }
            let rest: Refs = src.rest.iter().map(|c| c.with(&r.events)).collect();
            let rest = self.copied(&rest, op, 0);
            let target = self.state.heap.get_mut(&id).unwrap();
            target.fields = fields;
            target.items = items;
            target.rest = rest;
            target.length = match r.val {
                Val::Slice { length, .. } => length,
                _ => src.length,
            };
            target.opaque = src.opaque;
            target.type_name = src.type_name.clone();
            let val = match r.val {
                Val::Slice { length, .. } => Val::Slice {
                    backing: id,
                    offset: Some(0),
                    length,
                    capacity: length,
                },
                _ => Val::Obj { id, pointer: true },
            };
            out.insert(Ref::plain(val));
        }
        out
    }

    fn deep_copy(&mut self, refs: &Refs, span: Span) -> Refs {
        let mut out = Refs::new();
        for r in refs {
            let Some(object) = r.object() else {
                out.insert(Ref::plain(r.val.clone()));
                continue;
            };
            let Some(src) = self.state.heap.get(&object).cloned() else {
                continue;
            };
            let id = self.alloc(span, src.container, false);
            let mut fields = BTreeMap::new();
            for (name, children) in &src.fields {
                fields.insert(name.clone(), self.deep_copy(children, span));
            }
            let mut items = BTreeMap::new();
            for (index, children) in &src.items {
                items.insert(*index, self.deep_copy(children, span));
            }
            let rest = self.deep_copy(&src.rest, span);
            let target = self.state.heap.get_mut(&id).unwrap();
            target.fields = fields;
            target.items = items;
            target.rest = rest;
            target.length = src.length;
            target.opaque = src.opaque;
            out.insert(Ref::plain(Val::Obj { id, pointer: true }));
        }
        out
    }

    fn builtin_namespace(
        &mut self,
        namespace: &str,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> Refs {
        match (self.lang, namespace, name) {
            (Lang::JavaScript, "console", _)
            | (Lang::JavaScript, "JSON", "stringify")
            | (Lang::JavaScript, "Object", "keys" | "values" | "entries")
            | (Lang::JavaScript, "Array", "isArray")
            | (Lang::Go, "fmt" | "strings" | "strconv", _)
            | (Lang::Python, "json", "dumps") => {
                self.observe_all(args);
                Refs::new()
            }
            (Lang::JavaScript, "JSON", "parse") => {
                self.observe_all(args);
                let id = self.alloc(span, Some(Container::Object), false);
                Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
            }
            (Lang::JavaScript, "Object", "freeze") => {
                let refs = args
                    .first()
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                self.require(&refs, RequirementKind::Frozen, span);
                refs
            }
            (Lang::JavaScript, "Object", "assign") => {
                let Some(target_expr) = args.first() else {
                    return Refs::new();
                };
                let fresh = matches!(target_expr, Expr::Literal { .. });
                let target = self.eval(target_expr, Mode::Alias);
                let op = self.op(OperationKind::Call, span, None);
                for source in &args[1..] {
                    let refs = self.eval(source, Mode::Alias);
                    for r in &refs {
                        let Some(object) = r.object() else { continue };
                        let Some(src) = self.state.heap.get(&object).cloned() else {
                            continue;
                        };
                        for (field, children) in &src.fields {
                            let children: Refs =
                                children.iter().map(|c| c.with(&r.events)).collect();
                            let value = if fresh {
                                self.copied(&children, op, 0)
                            } else {
                                children
                            };
                            if fresh {
                                for t in &target {
                                    if let Some(t) = t.object() {
                                        self.state
                                            .heap
                                            .get_mut(&t)
                                            .unwrap()
                                            .fields
                                            .insert(field.clone(), value.clone());
                                    }
                                }
                            } else {
                                let path = self.path(target_expr).map(|mut p| {
                                    p.segments.push(Segment::Field(field.clone()));
                                    p
                                });
                                self.write_member(
                                    &target,
                                    Slot::Field(field.clone()),
                                    value,
                                    span,
                                    path,
                                    false,
                                );
                            }
                        }
                    }
                }
                target
            }
            (Lang::JavaScript, "Array", "from") => {
                let source = args
                    .first()
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                match args.get(1) {
                    None => self.shallow_copy(&source, span),
                    Some(callback) => {
                        let length = source.iter().find_map(|r| r.object()).and_then(|o| {
                            self.state
                                .heap
                                .get(&o)
                                .and_then(|o| o.fields.get("length"))
                                .and_then(|refs| match refs.iter().next().map(|r| &r.val) {
                                    Some(Val::Int(v)) if *v >= 0 => Some(*v as u64),
                                    _ => None,
                                })
                        });
                        let funcs = self.eval(callback, Mode::Alias);
                        let id = self.alloc(span, Some(Container::Array), false);
                        let func = funcs.iter().find_map(|r| match r.val {
                            Val::Func(f) => Some(f),
                            _ => None,
                        });
                        match func {
                            Some(f) => {
                                let unroll = length.unwrap_or(UNKNOWN_COUNT_UNROLL).min(MAX_UNROLL);
                                for i in 0..unroll {
                                    let call = self.op(OperationKind::Call, span, None);
                                    let refs = self.invoke(
                                        f,
                                        None,
                                        Args::positional(vec![
                                            Refs::new(),
                                            Refs::from([Ref::plain(Val::Int(i as i64))]),
                                        ]),
                                        call,
                                        false,
                                    );
                                    self.state.heap.get_mut(&id).unwrap().items.insert(i, refs);
                                }
                                self.state.heap.get_mut(&id).unwrap().length = length;
                            }
                            None => {
                                let b = self.boundary(
                                    span,
                                    "unknown-call",
                                    "Array.from mapping function is not resolved.",
                                );
                                self.state.heap.get_mut(&id).unwrap().rest =
                                    Refs::from([Ref::plain(Val::Unknown(b))]);
                            }
                        }
                        Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
                    }
                }
            }
            (Lang::Python, "copy", "copy") | (Lang::Go, "maps" | "slices", "Clone") => {
                let refs = args
                    .first()
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                self.shallow_copy(&refs, span)
            }
            (Lang::Python, "copy", "deepcopy") => {
                let refs = args
                    .first()
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                self.deep_copy(&refs, span)
            }
            _ => self.unknown_call(
                args,
                None,
                span,
                "External module function is not modeled; argument effects are unknown.",
            ),
        }
    }

    fn builtin_function(&mut self, name: &str, args: &[Expr], span: Span) -> Refs {
        match (self.lang, name) {
            (Lang::JavaScript, "structuredClone") => {
                let refs = args
                    .first()
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                self.deep_copy(&refs, span)
            }
            (Lang::JavaScript, "String" | "Number" | "Boolean" | "parseInt" | "parseFloat")
            | (
                Lang::Python,
                "print" | "str" | "repr" | "sum" | "min" | "max" | "any" | "all" | "isinstance"
                | "hash" | "id" | "bool" | "int" | "float" | "range" | "type",
            )
            | (Lang::Go, "print" | "println" | "string") => {
                self.observe_all(args);
                Refs::new()
            }
            (Lang::Python, "list" | "dict" | "set" | "tuple" | "sorted" | "frozenset") => {
                if args.is_empty() {
                    let container = match name {
                        "dict" => Container::Map,
                        "set" | "frozenset" => Container::Set,
                        _ => Container::Array,
                    };
                    let id = self.alloc(span, Some(container), false);
                    if container == Container::Array {
                        self.state.heap.get_mut(&id).unwrap().length = Some(0);
                    }
                    return Refs::from([Ref::plain(Val::Obj { id, pointer: true })]);
                }
                let refs = self.eval(&args[0], Mode::Alias);
                for a in &args[1..] {
                    self.eval(a, Mode::Use);
                }
                self.shallow_copy(&refs, span)
            }
            (Lang::Go, "len" | "cap") | (Lang::Python, "len") | (Lang::JavaScript, "isNaN") => {
                for a in args {
                    self.eval(a, Mode::Base);
                }
                Refs::new()
            }
            (Lang::Go, "append") => self.append(args, span),
            (Lang::Go, "make") => {
                let length = args.first().and_then(|a| self.integer(a));
                let capacity = match args.get(1) {
                    Some(a) => self.integer(a),
                    None => length,
                };
                let id = self.alloc(span, Some(Container::Array), false);
                self.state.heap.get_mut(&id).unwrap().length = capacity;
                Refs::from([Ref::plain(Val::Slice {
                    backing: id,
                    offset: Some(0),
                    length,
                    capacity,
                })])
            }
            (Lang::Go, "new") => {
                let id = self.alloc(span, Some(Container::Struct), true);
                Refs::from([Ref::plain(Val::Obj { id, pointer: true })])
            }
            (Lang::Go, "delete") => {
                let target = args
                    .first()
                    .map(|a| self.eval(a, Mode::Base))
                    .unwrap_or_default();
                let slot = match args.get(1).and_then(|a| self.literal_key(a)) {
                    Some(k) => Slot::Field(k),
                    None => {
                        if let Some(a) = args.get(1) {
                            self.eval(a, Mode::Use);
                        }
                        Slot::AnyField
                    }
                };
                let path = args.first().and_then(|a| self.path(a)).map(|mut p| {
                    p.segments.push(match &slot {
                        Slot::Field(k) => Segment::Field(k.clone()),
                        _ => Segment::Field("*".into()),
                    });
                    p
                });
                self.write_member(&target, slot, Refs::new(), span, path, false);
                Refs::new()
            }
            (Lang::Go, "copy") => {
                let target = args
                    .first()
                    .map(|a| self.eval(a, Mode::Base))
                    .unwrap_or_default();
                let source = args
                    .get(1)
                    .map(|a| self.eval(a, Mode::Alias))
                    .unwrap_or_default();
                let value = self.element_union(&source);
                let path = args.first().and_then(|a| self.path(a)).map(|mut p| {
                    p.segments.push(Segment::Element(None));
                    p
                });
                self.write_member(&target, Slot::AnyElement, value, span, path, false);
                Refs::new()
            }
            (Lang::Go, "panic") => {
                self.observe_all(args);
                self.throw_state();
                Refs::new()
            }
            _ => self.unknown_call(
                args,
                None,
                span,
                "Global function is not modeled; argument effects are unknown.",
            ),
        }
    }

    fn append(&mut self, args: &[Expr], span: Span) -> Refs {
        let Some(first) = args.first() else {
            return Refs::new();
        };
        let slices = self.eval(first, Mode::Alias);
        let mut values = Refs::new();
        let mut count = Some(0u64);
        for a in &args[1..] {
            let refs = self.eval(a, Mode::Alias);
            if matches!(a, Expr::Variadic(..)) {
                count = None;
            }
            count = count.map(|c| c + 1);
            union(&mut values, &refs);
        }
        let mut out = Refs::new();
        for r in slices {
            let Val::Slice {
                backing,
                offset,
                length,
                capacity,
            } = r.val
            else {
                out.insert(r);
                continue;
            };
            let fits = match (length, capacity, count) {
                (Some(l), Some(c), Some(n)) => Some(l + n <= c),
                _ => None,
            };
            if fits != Some(false) {
                let in_place = Refs::from([Ref {
                    val: Val::Slice {
                        backing,
                        offset,
                        length,
                        capacity,
                    },
                    events: r.events.clone(),
                }]);
                let slot = match (length, count) {
                    (Some(l), Some(1)) => Slot::Element(l),
                    _ => Slot::AnyElement,
                };
                let path = self.path(first).map(|mut p| {
                    p.segments.push(Segment::Element(match slot {
                        Slot::Element(i) => Some(i),
                        _ => None,
                    }));
                    p
                });
                let before = self.loop_depth;
                if fits.is_none() {
                    self.loop_depth += 1;
                }
                self.write_member(&in_place, slot, values.clone(), span, path, true);
                self.loop_depth = before;
                out.insert(Ref {
                    val: Val::Slice {
                        backing,
                        offset,
                        length: length.zip(count).map(|(l, n)| l + n),
                        capacity,
                    },
                    events: r.events.clone(),
                });
            }
            if fits != Some(true) {
                let id = self.alloc(span, Some(Container::Array), false);
                let elements = self.element(backing, None);
                let target = self.state.heap.get_mut(&id).unwrap();
                target.rest = elements;
                union(&mut target.rest, &values);
                out.insert(Ref::plain(Val::Slice {
                    backing: id,
                    offset: Some(0),
                    length: length.zip(count).map(|(l, n)| l + n),
                    capacity: None,
                }));
            }
        }
        out
    }

    fn throw_state(&mut self) {
        if let Some(t) = self.throws.last_mut() {
            t.push(self.state.clone());
        }
        self.state.terminated = true;
    }

    fn assign_stmt(&mut self, stmt: &Stmt) -> Refs {
        self.exec(stmt);
        match stmt {
            Stmt::Assign { target, .. } => self.peek(target),
            _ => Refs::new(),
        }
    }

    fn peek(&self, target: &Expr) -> Refs {
        match target {
            Expr::Var {
                key: Key::Binding(decl),
                ..
            } => self
                .lookup(*decl)
                .map(|(_, r)| r.clone())
                .unwrap_or_default(),
            _ => Refs::new(),
        }
    }

    fn block(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            if self.state.terminated || self.exhausted {
                break;
            }
            self.exec(stmt);
        }
    }

    fn exec(&mut self, stmt: &Stmt) {
        if !self.step() {
            return;
        }
        match stmt {
            Stmt::Bind {
                key,
                name,
                span,
                statement,
                value,
                requirement,
            } => {
                let boundaries = self.graph.heap.boundaries.len();
                let refs = match value {
                    Some(v) => {
                        let refs = self.eval(v, Mode::Alias);
                        self.materialize(refs, v.span())
                    }
                    None => Refs::new(),
                };
                if let Some((kind, at)) = requirement {
                    self.require(&refs, *kind, *at);
                }
                for r in &refs {
                    if let Val::Func(index) = r.val {
                        self.closures[index]
                            .name
                            .get_or_insert_with(|| name.clone());
                    }
                }
                if let Key::Binding(decl) = key {
                    let context = self.context();
                    if context == 0 {
                        if let (Some(r), Some(v)) = (refs.iter().next(), value) {
                            if let (Val::Obj { id, .. }, 1) = (&r.val, refs.len()) {
                                self.graph.heap.bindings.push(Binding {
                                    name: name.clone(),
                                    object: *id,
                                    range: v.span(),
                                });
                            }
                        }
                    }
                    self.state.env.insert((*decl, context), refs);
                }
                let _ = span;
                if self.graph.heap.boundaries.len() == boundaries {
                    self.graph.heap.modeled.push(*statement);
                }
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                let boundaries = self.graph.heap.boundaries.len();
                let refs = self.eval(value, Mode::Alias);
                let refs = self.materialize(refs, value.span());
                match target {
                    Expr::Var {
                        key: Key::Binding(decl),
                        ..
                    } => {
                        let context = self.lookup(*decl).map_or(self.context(), |(c, _)| c);
                        self.state.env.insert((*decl, context), refs);
                    }
                    Expr::Field {
                        base,
                        name,
                        span: at,
                    } => {
                        let b = self.eval(base, Mode::Base);
                        self.write_member(
                            &b,
                            Slot::Field(name.clone()),
                            refs,
                            *at,
                            self.path(target),
                            false,
                        );
                    }
                    Expr::Index {
                        base,
                        index,
                        span: at,
                    } => {
                        let b = self.eval(base, Mode::Base);
                        let slot = self.slot(&b, index, *at);
                        self.write_member(&b, slot, refs, *at, self.path(target), false);
                    }
                    other => {
                        self.eval(other, Mode::Use);
                    }
                }
                if self.graph.heap.boundaries.len() == boundaries {
                    self.graph.heap.modeled.push(*span);
                }
            }
            Stmt::Eval(expr) => {
                self.eval(expr, Mode::Alias);
            }
            Stmt::If {
                condition,
                then,
                otherwise,
            } => {
                self.guard_depth += 1;
                self.eval(condition, Mode::Use);
                self.guard_depth -= 1;
                let before = self.state.clone();
                self.block(then);
                let after_then = std::mem::replace(&mut self.state, before);
                self.block(otherwise);
                let after_else = std::mem::take(&mut self.state);
                self.state = join(after_then, after_else);
            }
            Stmt::Return { value, span } => {
                let refs = match value {
                    Some(v) => {
                        let refs = self.eval(v, Mode::Alias);
                        let refs = self.materialize(refs, v.span());
                        self.returned(refs, v, *span)
                    }
                    None => Refs::new(),
                };
                if let Some(frame) = self.frames.last_mut() {
                    frame.returns.push((self.state.clone(), refs));
                }
                self.state.terminated = true;
            }
            Stmt::Throw { value, .. } => {
                if let Some(v) = value {
                    self.eval(v, Mode::Use);
                }
                self.throw_state();
            }
            Stmt::Loop { header, body, .. } => {
                self.limit(LIMIT_LOOPS);
                self.loop_depth += 1;
                self.block(header);
                let before = self.state.clone();
                self.block(body);
                let after = std::mem::take(&mut self.state);
                self.loop_depth -= 1;
                let mut joined = join(before, after);
                joined.terminated = false;
                self.state = joined;
            }
            Stmt::Try {
                body,
                handler,
                finally,
                ..
            } => {
                self.limit(LIMIT_EXCEPTIONS);
                let before = self.state.clone();
                self.try_depth += 1;
                self.throws.push(vec![]);
                self.block(body);
                let thrown = self.throws.pop().unwrap_or_default();
                self.try_depth -= 1;
                let body_end = self.state.clone();
                let mut entry = before;
                let mut normal = body_end.clone();
                normal.terminated = false;
                entry = join(entry, normal);
                for state in thrown {
                    let mut state = state;
                    state.terminated = false;
                    entry = join(entry, state);
                }
                if !handler.is_empty() {
                    self.state = entry;
                    self.try_depth += 1;
                    self.block(handler);
                    self.try_depth -= 1;
                    let after = std::mem::take(&mut self.state);
                    self.state = join(body_end, after);
                } else {
                    self.state = body_end;
                }
                self.block(finally);
            }
            Stmt::Method {
                type_name,
                name,
                func,
            } => {
                let refs = self.closure(func.clone());
                if let Some(Val::Func(index)) = refs.iter().next().map(|r| r.val.clone()) {
                    self.methods
                        .insert((type_name.clone(), name.clone()), index);
                }
            }
            Stmt::Unsupported {
                reason,
                span,
                binds,
            } => {
                let b = self.boundary(*span, "unsupported-statement", reason);
                for (key, _, _) in binds {
                    if let Key::Binding(decl) = key {
                        let context = self.context();
                        self.state
                            .env
                            .insert((*decl, context), Refs::from([Ref::plain(Val::Unknown(b))]));
                    }
                }
            }
        }
    }
}
