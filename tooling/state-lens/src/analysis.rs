use crate::model::*;
use anyhow::{bail, Context, Result};
use std::sync::{Arc, OnceLock};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use tree_sitter::{
    InputEdit, Language, Node, Parser, Point, Query, QueryCursor, StreamingIterator, Tree,
};
use url::Url;
fn rule_metadata() -> &'static serde_json::Value {
    static METADATA: OnceLock<serde_json::Value> = OnceLock::new();
    METADATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../queries/rules.json")).expect("rule metadata")
    })
}
fn related_cwes(rule_id: &str) -> Vec<String> {
    rule_metadata()["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == rule_id)
        .and_then(|r| r["related_cwes"].as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
include!(concat!(env!("OUT_DIR"), "/queries.rs"));

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const EXCLUDED: &[&str] = &["node_modules", "target", "vendor", "dist", "build", ".git"];
const MUTATING_METHODS: &[&str] = &[
    "sort",
    "sort_unstable",
    "reverse",
    "push",
    "pop",
    "splice",
    "shift",
    "unshift",
    "set",
    "insert",
    "remove",
    "clear",
    "append",
    "truncate",
    "retain",
    "borrow_mut",
    "fetch_add",
    "fetch_sub",
    "store",
];

pub fn language_name(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()? {
        "go" => Some("go"),
        "rs" => Some("rust"),
        "ts" => Some("typescript"),
        "py" => Some("python"),
        _ => None,
    }
}
fn grammar(name: &str) -> Result<Language> {
    Ok(match name {
        "go" => tree_sitter_go::LANGUAGE.into(),
        "python" => tree_sitter_python::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        _ => bail!("unsupported language: {name}"),
    })
}
fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}
fn nodes(node: Node<'_>) -> Vec<Node<'_>> {
    let mut result = vec![];
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        result.push(n);
        let mut cursor = n.walk();
        let children: Vec<_> = n.named_children(&mut cursor).collect();
        stack.extend(children.into_iter().rev());
    }
    result
}
fn point(source: &str, offset: usize) -> Point {
    let prefix = &source[..offset];
    Point::new(
        prefix.bytes().filter(|b| *b == b'\n').count(),
        prefix.rfind('\n').map_or(offset, |i| offset - i - 1),
    )
}
fn scope(node: Node<'_>, root: Span) -> Span {
    let mut ancestor = Some(node);
    let mut python = false;
    while let Some(n) = ancestor {
        python |= n.kind() == "module";
        ancestor = n.parent();
    }
    let mut current = node.parent();
    while let Some(n) = current {
        if python {
            if matches!(
                n.kind(),
                "function_definition" | "lambda" | "class_definition"
            ) {
                return Span::of(n);
            }
            current = n.parent();
            continue;
        }
        if matches!(
            n.kind(),
            "statement_block"
                | "block"
                | "for_statement"
                | "for_expression"
                | "function_definition"
                | "lambda"
                | "function_declaration"
                | "function_item"
                | "method_declaration"
                | "method_definition"
                | "arrow_function"
                | "function_expression"
                | "func_literal"
                | "closure_expression"
        ) {
            return Span::of(n);
        }
        current = n.parent();
    }
    root
}
fn owner(functions: &[Function], span: Span) -> Option<usize> {
    functions
        .iter()
        .enumerate()
        .filter(|(_, f)| f.range.contains(span))
        .min_by_key(|(_, f)| f.range.len())
        .map(|(i, _)| i)
}

#[derive(Clone)]
struct Binding {
    name: String,
    range: Span,
    scope: Span,
    function: Option<usize>,
    parameter: Option<usize>,
    mutable: bool,
    type_name: String,
    language: String,
    alias: Option<String>,
    fresh: bool,
    callable: Option<Span>,
}
#[derive(Clone)]
struct Call {
    range: Span,
    owner: Option<usize>,
    name: String,
    arguments: Vec<String>,
}
#[derive(Clone)]
struct Import {
    local: String,
    symbol: Option<String>,
    module: String,
}
struct Parsed {
    tree: Tree,
    source: String,
    language: String,
    version: Option<i64>,
}
#[derive(Clone)]
struct Facts {
    report: DocumentReport,
    bindings: Vec<Binding>,
    calls: Vec<Call>,
    imports: Vec<Import>,
    source: String,
    resolvable: Vec<bool>,
}

pub struct Engine {
    parsed: BTreeMap<String, Parsed>,
    queries: BTreeMap<String, Vec<(String, Query)>>,
    cache: BTreeMap<String, Facts>,
    pub generation: u64,
    pub skipped: Vec<String>,
    memo: Option<(u64, usize, Arc<Report>)>,
}
impl Engine {
    pub fn new() -> Result<Self> {
        let mut queries: BTreeMap<String, Vec<(String, Query)>> = BTreeMap::new();
        for &(path, source) in QUERIES {
            let language = path.split('/').next().unwrap();
            let query =
                Query::new(&grammar(language)?, source).with_context(|| format!("query {path}"))?;
            queries
                .entry(language.into())
                .or_default()
                .push((path.into(), query));
        }
        Ok(Self {
            parsed: BTreeMap::new(),
            queries,
            cache: BTreeMap::new(),
            generation: 0,
            skipped: vec![],
            memo: None,
        })
    }
    pub fn set(
        &mut self,
        uri: String,
        language: &str,
        source: String,
        version: Option<i64>,
    ) -> Result<()> {
        if source.len() as u64 > MAX_FILE_BYTES {
            bail!("document exceeds {MAX_FILE_BYTES} bytes");
        }
        let mut parser = Parser::new();
        parser.set_language(&grammar(language)?)?;
        let previous = self.parsed.get(&uri).filter(|p| p.language == language);
        let edited = previous.map(|old| {
            let mut tree = old.tree.clone();
            let mut start = old
                .source
                .bytes()
                .zip(source.bytes())
                .take_while(|(a, b)| a == b)
                .count();
            while !old.source.is_char_boundary(start) || !source.is_char_boundary(start) {
                start -= 1;
            }
            let mut suffix = old.source[start..]
                .bytes()
                .rev()
                .zip(source[start..].bytes().rev())
                .take_while(|(a, b)| a == b)
                .count();
            while !old.source.is_char_boundary(old.source.len() - suffix)
                || !source.is_char_boundary(source.len() - suffix)
            {
                suffix -= 1;
            }
            let old_end = old.source.len() - suffix;
            let new_end = source.len() - suffix;
            tree.edit(&InputEdit {
                start_byte: start,
                old_end_byte: old_end,
                new_end_byte: new_end,
                start_position: point(&old.source, start),
                old_end_position: point(&old.source, old_end),
                new_end_position: point(&source, new_end),
            });
            tree
        });
        let tree = parser
            .parse(&source, edited.as_ref())
            .context("parse cancelled")?;
        self.cache.remove(&uri);
        self.parsed.insert(
            uri,
            Parsed {
                tree,
                source,
                language: language.into(),
                version,
            },
        );
        self.generation += 1;
        Ok(())
    }
    pub fn remove(&mut self, uri: &str) {
        self.parsed.remove(uri);
        self.cache.remove(uri);
        self.generation += 1;
    }
    pub fn source(&self, uri: &str) -> Option<&str> {
        self.parsed.get(uri).map(|p| p.source.as_str())
    }
    pub fn workspace_files(path: &Path) -> Result<Vec<PathBuf>> {
        let root = path.canonicalize()?;
        let mut files = Vec::new();
        for entry in ignore::WalkBuilder::new(root)
            .hidden(true)
            .filter_entry(|e| !EXCLUDED.contains(&e.file_name().to_str().unwrap_or("")))
            .build()
        {
            let entry = entry?;
            if entry.file_type().is_some_and(|t| t.is_file())
                && language_name(entry.path()).is_some()
            {
                files.push(entry.into_path());
            }
        }
        files.sort();
        Ok(files)
    }
    pub fn load(&mut self, path: &Path) -> Result<()> {
        let absolute = path.canonicalize()?;
        if absolute.is_file() {
            return self.load_file(&absolute);
        }
        for entry in ignore::WalkBuilder::new(&absolute)
            .hidden(true)
            .filter_entry(|e| !EXCLUDED.contains(&e.file_name().to_str().unwrap_or("")))
            .build()
        {
            match entry {
                Ok(e)
                    if e.file_type().is_some_and(|t| t.is_file())
                        && language_name(e.path()).is_some() =>
                {
                    if let Err(error) = self.load_file(e.path()) {
                        self.skipped
                            .push(format!("{}: {error}", e.path().display()));
                    }
                }
                Err(error) => self.skipped.push(error.to_string()),
                _ => {}
            }
        }
        Ok(())
    }
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let language = language_name(path).context("unsupported file extension")?;
        if path.metadata()?.len() > MAX_FILE_BYTES {
            bail!("file exceeds size limit");
        }
        let source = std::fs::read_to_string(path)?;
        if source
            .lines()
            .take(8)
            .any(|l| l.contains("Code generated") && l.contains("DO NOT EDIT"))
        {
            return Ok(());
        }
        let uri = Url::from_file_path(path)
            .map_err(|_| anyhow::anyhow!("invalid file path"))?
            .to_string();
        self.set(uri, language, source, None)
    }
    pub fn analyze(&mut self) -> Report {
        (*self.report()).clone()
    }
    pub fn report(&mut self) -> Arc<Report> {
        if let Some((generation, skipped, report)) = &self.memo {
            if *generation == self.generation && *skipped == self.skipped.len() {
                return report.clone();
            }
        }
        let report = Arc::new(self.compute());
        self.memo = Some((self.generation, self.skipped.len(), report.clone()));
        report
    }
    fn compute(&mut self) -> Report {
        for (uri, p) in &self.parsed {
            if !self.cache.contains_key(uri) {
                self.cache
                    .insert(uri.clone(), extract(uri, p, &self.queries[&p.language]));
            }
        }
        let mut facts: Vec<_> = self.cache.values().cloned().collect();
        propagate(&mut facts);
        Report { schema_version: SCHEMA_VERSION, generation: self.generation, documents: facts.into_iter().map(|mut f| {
            for function in &mut f.report.functions {
                function.effects = f.report.findings.iter().filter(|x| x.function == Some(function.range)).map(|x| x.effect.clone()).collect::<BTreeSet<_>>().into_iter().collect();
            }
            f.report.findings.sort_by_key(|x| (x.trigger.start,x.rule_id.clone(),x.origin.clone()));
            f.report
        }).collect(), limitations: std::iter::once("Susceptibility is not a bug diagnosis. Dynamic dispatch, general alias escape, macro expansion, type inference and domain invariants are not resolved.".into()).chain(self.skipped.clone()).collect() }
    }
}

fn binding<'a>(bindings: &'a [Binding], name: &str, at: Span) -> Option<&'a Binding> {
    bindings
        .iter()
        .filter(|b| {
            b.name == name
                && b.scope.contains(at)
                && (b.parameter.is_some()
                    || b.function.is_none()
                    || b.language == "typescript"
                    || b.language == "python"
                    || b.range.start <= at.start)
        })
        .min_by_key(|b| (b.scope.len(), usize::MAX - b.range.start))
}
fn base_name(expression: &str) -> &str {
    expression
        .trim()
        .trim_start_matches('&')
        .trim_start_matches("mut ")
        .trim_start_matches('*')
        .split(['.', '[', '('])
        .next()
        .unwrap_or("")
        .trim()
}
fn mutation(
    bindings: &[Binding],
    functions: &[Function],
    target: &str,
    at: Span,
    indirect: bool,
) -> Effect {
    let name = base_name(target);
    if name == "this" || name == "self" {
        return Effect::ReceiverMutation;
    }
    let Some(b) = binding(bindings, name, at) else {
        return Effect::StructuralSignal;
    };
    if b.function.is_some() && b.function != owner(functions, at) {
        return Effect::GlobalMutation;
    }
    if indirect {
        if let Some(index) = b.parameter {
            let ty = b.type_name.replace(' ', "");
            if b.language == "rust"
                && !ty.contains("&mut")
                && !ty.contains("Cell<")
                && !ty.contains("Atomic")
            {
                return Effect::LocalMutation;
            }
            if b.language == "go"
                && !(ty.starts_with('*') || ty.starts_with("[]") || ty.starts_with("map["))
            {
                return Effect::StructuralSignal;
            }
            return Effect::ArgumentMutation(index);
        }
        if let Some(alias) = &b.alias {
            if alias != name {
                if let Some(original) = binding(bindings, alias, b.range) {
                    if let Some(index) = original.parameter {
                        return Effect::ArgumentMutation(index);
                    }
                    if original.function.is_none() {
                        return Effect::GlobalMutation;
                    }
                }
            }
        }
    }
    if b.function.is_none() || b.function != owner(functions, at) {
        Effect::GlobalMutation
    } else {
        Effect::LocalMutation
    }
}
fn add(
    facts: &mut Facts,
    node: Node<'_>,
    defect: &str,
    effect: Effect,
    explanation: &str,
    resolution: &str,
) {
    let trigger = Span::of(node);
    let family = rule_metadata()["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rule| rule["defect"] == defect)
        .and_then(|rule| rule["family"].as_str())
        .expect("finding must have registered rule metadata");
    let rule_id = format!("{}.{}.{}", facts.report.language, family, defect);
    if facts
        .report
        .findings
        .iter()
        .any(|f| f.trigger == trigger && f.rule_id == rule_id && f.effect == effect)
    {
        return;
    }
    if matches!(resolution, "candidate")
        || effect == Effect::StructuralSignal && resolution == "binding-analysis"
    {
        let unresolved = Unresolved {
            range: trigger,
            reason: explanation.into(),
        };
        if !facts
            .report
            .unresolved
            .iter()
            .any(|u| u.range == unresolved.range && u.reason == unresolved.reason)
        {
            facts.report.unresolved.push(unresolved);
        }
    }
    facts.report.findings.push(Finding {
        rule_id: rule_id.clone(),
        family: family.into(),
        compound: None,
        defect: defect.into(),
        trigger,
        function: owner(&facts.report.functions, trigger).map(|i| facts.report.functions[i].range),
        effect,
        origin: "direct".into(),
        resolution: resolution.into(),
        explanation: explanation.into(),
        related_cwes: related_cwes(&rule_id),
        evidence: vec![Evidence {
            uri: facts.report.uri.clone(),
            range: trigger,
            rule_id,
        }],
        call_path: vec![],
    });
}

fn extract(uri: &str, parsed: &Parsed, queries: &[(String, Query)]) -> Facts {
    let source = &parsed.source;
    let root = parsed.tree.root_node();
    let all = nodes(root);
    let mut captures = vec![];
    for (path, query) in queries {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, root, source.as_bytes());
        while let Some(m) = matches.next() {
            for c in m.captures {
                captures.push((
                    path.as_str(),
                    query.capture_names()[c.index as usize],
                    c.node,
                ));
            }
        }
    }
    let mut functions = vec![];
    for (_, capture, node) in &captures {
        if *capture != "scope" {
            continue;
        }
        let name = node
            .child_by_field_name("name")
            .or_else(|| {
                node.parent()
                    .filter(|p| p.kind() == "variable_declarator")
                    .and_then(|p| p.child_by_field_name("name"))
            })
            .map(|n| text(n, source).to_string())
            .unwrap_or_else(|| format!("<closure@{}>", node.start_byte()));
        let body = node
            .child_by_field_name("body")
            .map(Span::of)
            .unwrap_or(Span::of(*node));
        functions.push(Function {
            name,
            range: Span::of(*node),
            body,
            parameters: vec![],
            effects: vec![],
        });
    }
    functions.sort_by_key(|f| f.range.start);
    functions.dedup_by_key(|f| f.range);
    let mut facts = Facts {
        report: DocumentReport {
            uri: uri.into(),
            language: parsed.language.clone(),
            source_hash: hash(source),
            version: parsed.version,
            functions,
            findings: vec![],
            unresolved: vec![],
            parse_errors: root.has_error(),
        },
        bindings: vec![],
        calls: vec![],
        imports: vec![],
        source: source.clone(),
        resolvable: vec![],
    };
    for function in &facts.report.functions {
        let node = all
            .iter()
            .find(|n| Span::of(**n) == function.range)
            .unwrap();
        facts.resolvable.push(
            !(matches!(node.kind(), "method_declaration" | "method_definition")
                || (node.kind() == "function_item"
                    && node
                        .parent()
                        .is_some_and(|p| p.kind() == "declaration_list"))),
        );
    }
    if parsed.language == "python" {
        for (index, function) in facts.report.functions.iter_mut().enumerate() {
            let node = all
                .iter()
                .find(|n| Span::of(**n) == function.range)
                .unwrap();
            if let Some(parameters) = node.child_by_field_name("parameters") {
                let mut cursor = parameters.walk();
                for parameter in parameters.named_children(&mut cursor) {
                    let name = if parameter.kind() == "identifier" {
                        Some(parameter)
                    } else {
                        parameter.child_by_field_name("name").or_else(|| {
                            parameter
                                .named_child(0)
                                .filter(|n| n.kind() == "identifier")
                        })
                    };
                    if let Some(name) = name {
                        let value = text(name, source).to_string();
                        facts.bindings.push(Binding {
                            name: value.clone(),
                            range: Span::of(name),
                            scope: function.range,
                            function: Some(index),
                            parameter: Some(function.parameters.len()),
                            mutable: true,
                            type_name: String::new(),
                            language: parsed.language.clone(),
                            alias: None,
                            fresh: false,
                            callable: None,
                        });
                        function.parameters.push(value);
                    }
                }
            }
        }
    }
    for n in &all {
        let span = Span::of(*n);
        if matches!(
            n.kind(),
            "required_parameter"
                | "optional_parameter"
                | "parameter"
                | "parameter_declaration"
                | "variadic_parameter_declaration"
        ) {
            let Some(index) = owner(&facts.report.functions, span) else {
                continue;
            };
            let names: Vec<_> = if parsed.language == "go" {
                let mut c = n.walk();
                n.children_by_field_name("name", &mut c).collect()
            } else {
                n.child_by_field_name("pattern")
                    .or_else(|| n.child_by_field_name("name"))
                    .into_iter()
                    .collect()
            };
            for name in names.into_iter().filter_map(|n| {
                if n.kind() == "identifier" {
                    Some(n)
                } else if n.kind() == "mut_pattern" {
                    nodes(n)
                        .into_iter()
                        .find(|part| part.kind() == "identifier")
                } else {
                    None
                }
            }) {
                let parameter = facts.report.functions[index].parameters.len();
                let value = text(name, source).to_string();
                facts.report.functions[index].parameters.push(value.clone());
                facts.bindings.push(Binding {
                    name: value,
                    range: Span::of(name),
                    scope: facts.report.functions[index].range,
                    function: Some(index),
                    parameter: Some(parameter),
                    mutable: true,
                    language: parsed.language.clone(),
                    type_name: n
                        .child_by_field_name("type")
                        .map(|t| text(t, source).to_string())
                        .unwrap_or_default(),
                    alias: None,
                    fresh: false,
                    callable: None,
                });
            }
        }
        if matches!(
            n.kind(),
            "variable_declarator"
                | "let_declaration"
                | "var_spec"
                | "const_spec"
                | "short_var_declaration"
                | "static_item"
                | "assignment"
        ) {
            let name = n
                .child_by_field_name("name")
                .or_else(|| n.child_by_field_name("pattern"))
                .or_else(|| n.child_by_field_name("left"));
            let value = n
                .child_by_field_name("value")
                .or_else(|| n.child_by_field_name("right"));
            if let Some(name) = name {
                let names: Vec<_> = if name.kind() == "expression_list" {
                    let mut c = name.walk();
                    name.named_children(&mut c).collect()
                } else {
                    vec![name]
                };
                for name in names.into_iter().filter(|n| n.kind() == "identifier") {
                    if parsed.language == "python" {
                        let function = owner(&facts.report.functions, span);
                        let declared_outer = all.iter().any(|declaration| {
                            matches!(
                                declaration.kind(),
                                "global_statement" | "nonlocal_statement"
                            ) && owner(&facts.report.functions, Span::of(*declaration)) == function
                                && nodes(*declaration).iter().any(|part| {
                                    part.kind() == "identifier"
                                        && text(*part, source) == text(name, source)
                                })
                        });
                        if declared_outer {
                            continue;
                        }
                    }
                    let value = value.and_then(|v| {
                        if v.kind() == "expression_list" && v.named_child_count() == 1 {
                            v.named_child(0)
                        } else {
                            Some(v)
                        }
                    });
                    let alias = value
                        .filter(|v| v.kind() == "identifier")
                        .map(|v| text(v, source).into());
                    let fresh = value.is_some_and(|v| {
                        matches!(
                            v.kind(),
                            "array"
                                | "object"
                                | "array_expression"
                                | "struct_expression"
                                | "composite_literal"
                                | "integer_literal"
                                | "number"
                                | "int_literal"
                                | "string"
                                | "string_literal"
                        )
                    });
                    let mutable = match parsed.language.as_str() {
                        "rust" => {
                            nodes(*n).iter().any(|x| x.kind() == "mutable_specifier")
                                || n.kind() == "static_item"
                        }
                        "typescript" => {
                            n.parent().is_some_and(|p| {
                                p.kind() == "variable_declaration"
                                    || text(p, source).starts_with("let ")
                            }) || value.is_some_and(|v| matches!(v.kind(), "array" | "object"))
                        }
                        _ => n.kind() != "const_spec",
                    };
                    facts.bindings.push(Binding {
                        name: text(name, source).into(),
                        range: Span::of(name),
                        scope: scope(*n, Span::of(root)),
                        function: owner(&facts.report.functions, span),
                        parameter: None,
                        mutable,
                        language: parsed.language.clone(),
                        type_name: n
                            .child_by_field_name("type")
                            .map(|t| text(t, source).to_string())
                            .unwrap_or_default(),
                        alias,
                        fresh,
                        callable: value
                            .filter(|v| {
                                matches!(
                                    v.kind(),
                                    "arrow_function"
                                        | "function_expression"
                                        | "func_literal"
                                        | "closure_expression"
                                )
                            })
                            .map(Span::of),
                    });
                }
            }
        }
        if matches!(
            n.kind(),
            "function_definition"
                | "function_declaration"
                | "function_item"
                | "class_declaration"
                | "struct_item"
        ) {
            if let Some(name) = n.child_by_field_name("name") {
                facts.bindings.push(Binding {
                    name: text(name, source).into(),
                    range: Span::of(name),
                    scope: scope(*n, Span::of(root)),
                    function: None,
                    parameter: None,
                    mutable: false,
                    language: parsed.language.clone(),
                    type_name: String::new(),
                    alias: None,
                    fresh: false,
                    callable: facts
                        .report
                        .functions
                        .iter()
                        .find(|f| f.range == span)
                        .map(|f| f.range),
                });
            }
        }
        extract_import(*n, source, &parsed.language, &mut facts.imports);
    }
    for (path, capture, node) in captures {
        if node.has_error() || node.is_missing() {
            continue;
        }
        let defect = path.split('/').nth(2).unwrap_or("");
        let at = Span::of(node);
        match capture {
            "binding" => add(&mut facts,node,defect,Effect::LocalMutation,"Mutable binding; local mutation does not imply observable impurity.","syntax"),
            "write"|"update"|"receiver" => {
                let target=node.child_by_field_name("left").or_else(||node.child_by_field_name("argument")).or_else(||node.named_child(0));
                if let Some(target)=target {
                    let value=text(target,source);
                    let indirect=value.contains('.') || value.contains('[') || value.starts_with('*');
                    let effect=mutation(&facts.bindings,&facts.report.functions,value,at,indirect);
                    add(&mut facts,node,defect,effect,"Write to a binding or referenced value.","binding-analysis");
                }
            }
            "alias" => add(&mut facts,node,defect,Effect::StructuralSignal,"Copy or clone may retain aliases; type and escape behavior are not proven.","candidate"),
            "loop" => {
                let has_write=all.iter().any(|n| at.contains(Span::of(*n)) && owner(&facts.report.functions,Span::of(*n))==owner(&facts.report.functions,at) && matches!(n.kind(),"assignment_expression"|"augmented_assignment_expression"|"assignment_statement"|"compound_assignment_expr"|"update_expression"|"inc_statement"|"dec_statement"|"assignment"|"augmented_assignment"));
                if has_write {add(&mut facts,node,defect,Effect::StructuralSignal,"Loop contains state updates; ordering and iteration boundaries require review.","syntax");}
            }
            "receiver_signature" => add(&mut facts,node,defect,Effect::StructuralSignal,"Receiver permits mutation; signature alone does not prove a write.","syntax"),
            "numeric_cast" => {
                if let Some(callee)=node.child_by_field_name("function") {
                    if binding(&facts.bindings,text(callee,source),at).is_none(){add(&mut facts,node,defect,Effect::StructuralSignal,"Numeric conversion may narrow or reinterpret a value.","syntax");}
                }
            }
            "interior" => add(&mut facts,node,defect,Effect::StructuralSignal,"Receiver field uses a potential interior-mutation method; field type is unresolved.","candidate"),
            "numeric" => add(&mut facts,node,defect,Effect::StructuralSignal,"Explicit narrowing, wrapping, or fixed-width arithmetic mechanism.","syntax"),
            "initialization" => add(&mut facts,node,defect,Effect::StructuralSignal,"Default or asserted initialization may require a domain invariant.","candidate"),
            "macro"|"embedding" => add(&mut facts,node,defect,Effect::StructuralSignal,"Macro or embedding mechanism; expansion and dispatch behavior are not resolved.","candidate"),
            "mixin" if binding(&facts.bindings,"Object",at).is_none() => {
                add(&mut facts,node,defect,Effect::StructuralSignal,"Object.assign mutates its target and may overwrite existing properties.","syntax");
            }
            "call" => {
                let Some(callee)=node.child_by_field_name("function") else {continue;};
                let args=node.child_by_field_name("arguments").map(|a| {let mut c=a.walk();a.named_children(&mut c).map(|n|text(n,source).to_string()).collect()}).unwrap_or_default();
                facts.calls.push(Call {range:at,owner:owner(&facts.report.functions,at),name:text(callee,source).into(),arguments:args});
                model_call(&mut facts,node,callee,source);
            }
            "reference" => {
                if parsed.language == "python" && node.parent().is_some_and(|parent| {
                    matches!(parent.kind(), "global_statement" | "nonlocal_statement" | "import_statement" | "import_from_statement" | "aliased_import" | "dotted_name")
                        || parent.kind() == "attribute" && parent.child_by_field_name("attribute") == Some(node)
                        || parent.kind() == "assignment" && parent.child_by_field_name("left") == Some(node)
                        || parent.kind() == "function_definition" && parent.child_by_field_name("name") == Some(node)
                }) { continue; }
                if let Some(b)=binding(&facts.bindings,text(node,source),at) {
                    if b.mutable && b.range != at && b.function != owner(&facts.report.functions,at) {
                        add(&mut facts,node,"impure-functions",Effect::ExternalRead,"Read of mutable state outside this executable scope.","binding-analysis");
                    }
                }
            }
            _=>{}
        }
    }
    if root.has_error() {
        facts.report.unresolved.push(Unresolved {
            range: Span::of(root),
            reason: "Syntax errors: results are partial.".into(),
        });
    }
    facts
}

fn extract_import(node: Node<'_>, source: &str, language: &str, imports: &mut Vec<Import>) {
    if language == "typescript" && node.kind() == "import_statement" {
        let Some(module) = node.child_by_field_name("source") else {
            return;
        };
        let module = text(module, source).trim_matches(['\'', '"']).to_string();
        for n in nodes(node) {
            if n.kind() == "import_specifier" {
                if let Some(name) = n.child_by_field_name("name") {
                    let alias = n.child_by_field_name("alias").unwrap_or(name);
                    imports.push(Import {
                        local: text(alias, source).into(),
                        symbol: Some(text(name, source).into()),
                        module: module.clone(),
                    });
                }
            } else if n.kind() == "namespace_import" {
                if let Some(name) = n.named_child(0) {
                    imports.push(Import {
                        local: text(name, source).into(),
                        symbol: None,
                        module: module.clone(),
                    });
                }
            }
        }
    } else if language == "go" && node.kind() == "import_spec" {
        if let Some(path) = node.child_by_field_name("path") {
            let module = text(path, source).trim_matches(['"', '`']);
            let local = node
                .child_by_field_name("name")
                .map(|n| text(n, source))
                .unwrap_or_else(|| module.rsplit('/').next().unwrap_or(module));
            imports.push(Import {
                local: local.into(),
                symbol: None,
                module: module.into(),
            });
        }
    } else if language == "rust" && node.kind() == "use_declaration" {
        if let Some(argument) = node.child_by_field_name("argument") {
            let value = text(argument, source);
            if !value.contains(['{', '*']) {
                let mut alias = value.split(" as ");
                let path = alias.next().unwrap();
                if let Some((module, symbol)) = path.rsplit_once("::") {
                    imports.push(Import {
                        local: alias.next().unwrap_or(symbol).into(),
                        symbol: Some(symbol.into()),
                        module: module.into(),
                    });
                }
            }
        }
    }
}

fn model_call(facts: &mut Facts, node: Node<'_>, callee: Node<'_>, source: &str) {
    let name = text(callee, source);
    let at = Span::of(node);
    let imported = |base: &str| {
        facts
            .imports
            .iter()
            .find(|i| i.local == base && binding(&facts.bindings, base, at).is_none())
            .map(|i| i.module.as_str())
    };
    let nondeterministic = match facts.report.language.as_str() {
        "typescript" => {
            matches!(name, "Date.now" | "Math.random")
                && binding(&facts.bindings, base_name(name), at).is_none()
        }
        "go" => name.split_once('.').is_some_and(|(base, method)| {
            matches!(
                (imported(base), method),
                (Some("time"), "Now")
                    | (
                        Some("math/rand" | "math/rand/v2"),
                        "Intn" | "Float64" | "Int"
                    )
            )
        }),
        "rust" => {
            matches!(
                name,
                "std::time::SystemTime::now" | "std::time::Instant::now"
            ) || name.split_once("::").is_some_and(|(base, method)| {
                method == "now"
                    && facts.imports.iter().any(|i| {
                        i.local == base
                            && i.module == "std::time"
                            && matches!(i.symbol.as_deref(), Some("SystemTime" | "Instant"))
                    })
                    && binding(&facts.bindings, base, at).is_none()
            })
        }
        _ => false,
    };
    if nondeterministic {
        add(
            facts,
            node,
            "time-money-entropy",
            Effect::Nondeterminism,
            "Clock or entropy read from a resolved standard API.",
            "modeled-api",
        );
        return;
    }
    let args = node.child_by_field_name("arguments");
    if facts.report.language == "go"
        && name.split_once('.').is_some_and(|(base, method)| {
            imported(base) == Some("sort")
                && matches!(
                    method,
                    "Ints" | "Strings" | "Float64s" | "Slice" | "SliceStable"
                )
        })
    {
        if let Some(target) = args.and_then(|a| a.named_child(0)) {
            let effect = mutation(
                &facts.bindings,
                &facts.report.functions,
                text(target, source),
                at,
                true,
            );
            add(
                facts,
                node,
                "impure-functions",
                effect,
                "Standard sort mutates its argument.",
                "modeled-api",
            );
            return;
        }
    }
    let object = callee
        .child_by_field_name("object")
        .or_else(|| callee.child_by_field_name("value"));
    let method = callee
        .child_by_field_name("property")
        .or_else(|| callee.child_by_field_name("field"));
    if let (Some(object), Some(method)) = (object, method) {
        if MUTATING_METHODS.contains(&text(method, source)) {
            let effect = mutation(
                &facts.bindings,
                &facts.report.functions,
                text(object, source),
                at,
                true,
            );
            // Receiver methods need types; retain a candidate without propagating a guessed effect.
            let known = binding(&facts.bindings, text(object, source), at).is_some_and(|b| {
                let ty = b.type_name.replace(' ', "");
                match facts.report.language.as_str() {
                    "typescript" => {
                        ty.ends_with("[]")
                            && matches!(
                                text(method, source),
                                "sort"
                                    | "reverse"
                                    | "push"
                                    | "pop"
                                    | "splice"
                                    | "shift"
                                    | "unshift"
                            )
                    }
                    "rust" => {
                        ((ty.contains("Vec<") || ty.contains("["))
                            && matches!(
                                text(method, source),
                                "sort"
                                    | "sort_unstable"
                                    | "reverse"
                                    | "push"
                                    | "pop"
                                    | "insert"
                                    | "remove"
                                    | "clear"
                                    | "append"
                                    | "truncate"
                                    | "retain"
                            ))
                            || (ty.contains("Atomic")
                                && matches!(
                                    text(method, source),
                                    "fetch_add" | "fetch_sub" | "store"
                                ))
                            || (ty.contains("Cell<") && text(method, source) == "set")
                    }
                    _ => false,
                }
            });
            add(
                facts,
                node,
                "impure-functions",
                if known {
                    effect
                } else {
                    Effect::StructuralSignal
                },
                if known {
                    "Modeled mutation of an explicitly typed receiver."
                } else {
                    "Potential mutating method; receiver type is not resolved."
                },
                if known { "modeled-api" } else { "candidate" },
            );
        }
    }
}

fn uri_path(uri: &str) -> Option<PathBuf> {
    Url::parse(uri).ok()?.to_file_path().ok()
}
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            _ => out.push(c.as_os_str()),
        }
    }
    out
}
enum Target<'a> {
    Files(Vec<PathBuf>),
    Package(PathBuf, Option<&'a str>),
    Directory(PathBuf),
}
fn resolve(
    facts: &[Facts],
    paths: &[Option<PathBuf>],
    document: usize,
    call: &Call,
) -> Option<(usize, usize)> {
    let local = &facts[document];
    if !call.name.contains(['.', ':', '(', '[']) {
        if let Some(b) = binding(&local.bindings, &call.name, call.range) {
            let range = b.callable?;
            let index = local
                .report
                .functions
                .iter()
                .position(|f| f.range == range)?;
            return local.resolvable[index].then_some((document, index));
        }
    }
    let path = paths[document].clone()?;
    let mut target_module = None;
    let mut target_symbol = call.name.as_str();
    for import in &local.imports {
        if binding(&local.bindings, &import.local, call.range).is_some() {
            continue;
        }
        if import.symbol.is_some() && call.name == import.local {
            target_module = Some(import.module.as_str());
            target_symbol = import.symbol.as_deref()?;
            break;
        }
        if import.symbol.is_none() {
            if let Some(rest) = call.name.strip_prefix(&format!("{}.", import.local)) {
                target_module = Some(import.module.as_str());
                target_symbol = rest;
                break;
            }
        }
    }
    let target = match local.report.language.as_str() {
        "typescript" => {
            let module = target_module.filter(|m| m.starts_with('.'))?;
            let base = normalize(&path.parent()?.join(module));
            Target::Files(vec![base.with_extension("ts"), base.join("index.ts")])
        }
        "go" => match target_module {
            None if call.name.contains('.') => return None,
            None => Target::Package(path.parent()?.to_path_buf(), package(&local.source)),
            Some(module) => {
                let (root, name) = go_module(&path)?;
                let relative = module.strip_prefix(&format!("{name}/"))?;
                Target::Directory(root.join(relative))
            }
        },
        "rust" => {
            let parts: Vec<_> = target_module?.split("::").collect();
            if parts.first() != Some(&"crate") && parts.first() != Some(&"self") {
                return None;
            }
            let mut base = path.parent()?.to_path_buf();
            if parts[0] == "crate" {
                while !base.join("lib.rs").exists() && !base.join("main.rs").exists() {
                    if !base.pop() {
                        return None;
                    }
                }
            }
            for part in &parts[1..] {
                base.push(part);
            }
            Target::Files(vec![base.with_extension("rs"), base.join("mod.rs")])
        }
        _ => return None,
    };
    let mut targets = vec![];
    for (d, other) in facts.iter().enumerate() {
        if d == document || other.report.language != local.report.language {
            continue;
        }
        let Some(other_path) = paths[d].as_deref() else {
            continue;
        };
        let eligible = match &target {
            Target::Files(files) => files.iter().any(|f| f == other_path),
            Target::Package(directory, name) => {
                other_path.parent() == Some(directory.as_path()) && *name == package(&other.source)
            }
            Target::Directory(directory) => other_path.parent() == Some(directory.as_path()),
        };
        if eligible {
            for (f, function) in other.report.functions.iter().enumerate() {
                let is_top = other.resolvable[f]
                    && !other.report.functions.iter().any(|outer| {
                        outer.range != function.range && outer.range.contains(function.range)
                    });
                let exported = local.report.language != "typescript"
                    || other.source[..function.range.start]
                        .trim_end()
                        .ends_with("export")
                    || other.source[..function.range.start]
                        .trim_end()
                        .ends_with("export async");
                if function.name == target_symbol && is_top && exported {
                    targets.push((d, f));
                }
            }
        }
    }
    (targets.len() == 1).then(|| targets[0])
}
fn go_module(path: &Path) -> Option<(PathBuf, String)> {
    let mut directory = path.parent()?;
    loop {
        if let Ok(contents) = std::fs::read_to_string(directory.join("go.mod")) {
            let name = contents
                .lines()
                .find_map(|l| l.trim().strip_prefix("module "))?
                .trim()
                .trim_matches('"')
                .to_string();
            return Some((directory.to_path_buf(), name));
        }
        directory = directory.parent()?;
    }
}
fn package(source: &str) -> Option<&str> {
    source
        .lines()
        .find_map(|l| l.trim().strip_prefix("package "))
        .map(str::trim)
}

fn propagate(facts: &mut [Facts]) {
    let paths: Vec<_> = facts.iter().map(|f| uri_path(&f.report.uri)).collect();
    let mut edges = vec![];
    for (d, document) in facts.iter().enumerate() {
        for call in &document.calls {
            if let Some(target) = resolve(facts, &paths, d, call) {
                edges.push((d, call.clone(), target));
            }
        }
    }
    let resolved: BTreeSet<_> = edges.iter().map(|(d, c, _)| (*d, c.range)).collect();
    for (d, document) in facts.iter_mut().enumerate() {
        for call in &document.calls {
            if !resolved.contains(&(d, call.range))
                && !document.report.findings.iter().any(|f| {
                    f.trigger == call.range
                        && matches!(f.resolution.as_str(), "modeled-api" | "local-literal")
                })
            {
                document.report.unresolved.push(Unresolved {
                    range: call.range,
                    reason: format!("Unresolved call: {} (not assumed pure)", call.name),
                });
            }
        }
    }
    loop {
        let mut unknown = Vec::new();
        for (d, call, (target_doc, target_func)) in &edges {
            let target = &facts[*target_doc].report.functions[*target_func];
            if facts[*target_doc]
                .report
                .unresolved
                .iter()
                .any(|u| owner(&facts[*target_doc].report.functions, u.range) == Some(*target_func))
            {
                let reason = format!(
                    "Call to {} has unresolved downstream behavior.",
                    target.name
                );
                if !facts[*d]
                    .report
                    .unresolved
                    .iter()
                    .any(|u| u.range == call.range && u.reason == reason)
                {
                    unknown.push((
                        *d,
                        Unresolved {
                            range: call.range,
                            reason,
                        },
                    ));
                }
            }
        }
        if unknown.is_empty() {
            break;
        }
        for (d, unknown) in unknown {
            facts[d].report.unresolved.push(unknown);
        }
    }
    loop {
        let mut pending = vec![];
        for (d, call, (target_doc, target_func)) in &edges {
            let target = &facts[*target_doc].report.functions[*target_func];
            for finding in &facts[*target_doc].report.findings {
                if finding.function != Some(target.range) || !finding.effect.observable() {
                    continue;
                }
                let local = &facts[*d];
                if finding
                    .call_path
                    .iter()
                    .any(|e| e.uri == local.report.uri && e.range == call.range)
                {
                    continue;
                }
                let effect = if let Effect::ArgumentMutation(parameter) = finding.effect {
                    let Some(argument) = call.arguments.get(parameter) else {
                        continue;
                    };
                    let b = binding(&local.bindings, base_name(argument), call.range);
                    if b.is_some_and(|b| b.fresh && b.function == call.owner) {
                        Effect::LocalMutation
                    } else {
                        mutation(
                            &local.bindings,
                            &local.report.functions,
                            argument,
                            call.range,
                            true,
                        )
                    }
                } else if finding.effect == Effect::ReceiverMutation {
                    continue;
                } else {
                    finding.effect.clone()
                };
                if effect == Effect::StructuralSignal {
                    continue;
                }
                let mut propagated = finding.clone();
                propagated.trigger = call.range;
                propagated.function = call.owner.map(|i| local.report.functions[i].range);
                propagated.effect = effect;
                propagated.origin = "propagated".into();
                propagated.resolution = "resolved-call".into();
                propagated.explanation = format!(
                    "{} calls {}; {}",
                    call.owner
                        .map(|i| local.report.functions[i].name.as_str())
                        .unwrap_or("<module>"),
                    target.name,
                    finding.explanation
                );
                propagated.call_path.insert(
                    0,
                    Evidence {
                        uri: local.report.uri.clone(),
                        range: call.range,
                        rule_id: finding.rule_id.clone(),
                    },
                );
                if !local
                    .report
                    .findings
                    .iter()
                    .any(|f| same_effect(f, &propagated))
                    && !pending
                        .iter()
                        .any(|(doc, f)| *doc == *d && same_effect(f, &propagated))
                {
                    pending.push((*d, propagated));
                }
            }
        }
        if pending.is_empty() {
            break;
        }
        for (d, finding) in pending {
            facts[d].report.findings.push(finding);
        }
    }
}
fn same_effect(a: &Finding, b: &Finding) -> bool {
    a.trigger == b.trigger
        && a.function == b.function
        && a.effect == b.effect
        && a.evidence == b.evidence
        && a.origin == b.origin
}
