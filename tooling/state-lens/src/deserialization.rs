use crate::{annotations::nodes, defect_support::*, model::*, unstructured::add};
use tree_sitter::Node;

pub(crate) const DEFECT: &str = "insecure-deserialization";
const DECODE_BOUNDARY: &str = "decode-boundary";
const DECODED_SINK: &str = "decoded-to-sink";
const UNSAFE_LOADER: &str = "unsafe-loader";
const PROTOTYPE_KEY: &str = "prototype-key";
const UNTRUSTED_LENGTH: &str = "untrusted-length";
const UNCHECKED_DECODE: &str = "unchecked-decode";
const MAX_FLOW_DEPTH: usize = 8;
const FIELD_ASSIGNMENT: &str = "decoded-field-assignment";
const REVIVER: &str = "reviver-coercion";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Api {
    Json,
    GoJson,
    Pickle,
    Yaml,
}
impl Api {
    fn named(name: &str) -> Option<Self> {
        match name {
            "JSON.parse" | "json.loads" | "json.load" => Some(Self::Json),
            "json.Unmarshal" => Some(Self::GoJson),
            "pickle.loads" | "pickle.load" => Some(Self::Pickle),
            "yaml.load" => Some(Self::Yaml),
            _ => None,
        }
    }
}
fn decoder(node: Node<'_>, source: &str, resolve: &impl Fn(&str, Span) -> Option<Span>) -> bool {
    if callee(node).is_some_and(|c| resolve(base(c, source), Span::of(c)).is_some()) {
        return false;
    }
    callee(node).is_some_and(|c| Api::named(text(c, source)).is_some())
}

pub(crate) fn analyze(
    captures: &[(&str, &str, Node<'_>)],
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: impl Fn(&str, Span) -> Option<Span>,
) {
    let all = nodes(root);
    for &(path, capture, node) in captures {
        if !path.contains(DEFECT) || node.has_error() {
            continue;
        }
        let mut mechanism = None;
        let mut origins = vec![];
        if capture == "unstructured_decode" && calls(node) {
            let name = imported_call(node, &all, source, &resolve).unwrap_or_default();
            let api = Api::named(&name);
            let arguments = args(node);
            if callee(node).is_some_and(|c| resolve(base(c, source), Span::of(c)).is_some()) {
                continue;
            }
            if matches!(api, Some(Api::Pickle | Api::Yaml)) {
                let imported = all.iter().any(|n| {
                    matches!(n.kind(), "import_statement" | "import_from_statement")
                        && identifiers(*n)
                            .iter()
                            .any(|i| text(*i, source) == name.split('.').next().unwrap_or(""))
                });
                let safe_yaml = api == Some(Api::Yaml)
                    && arguments.iter().any(|n| {
                        field(*n, &["name"]).is_some_and(|name| text(name, source) == "Loader")
                            && field(*n, &["value"]).is_some_and(|loader| {
                                matches!(text(loader, source), "yaml.SafeLoader" | "SafeLoader")
                            })
                    });
                if imported && !safe_yaml {
                    mechanism = Some(UNSAFE_LOADER);
                }
            }
            if api == Some(Api::GoJson)
                && ancestor(node, &["assignment_statement"])
                    .and_then(left)
                    .is_some_and(|l| text(l, source) == "_")
            {
                mechanism = Some(UNCHECKED_DECODE);
            }
            if name == "JSON.parse" && arguments.len() > 1 && !filters_keys(arguments[1], source) {
                mechanism = Some(REVIVER);
            }
            if api == Some(Api::GoJson) && mechanism.is_none() {
                let target = arguments.get(1).map(|n| base(*n, source));
                let type_node = target.and_then(|target| {
                    all.iter()
                        .find(|n| {
                            n.kind() == "var_spec"
                                && field(**n, &["name"])
                                    .is_some_and(|name| text(name, source) == target)
                        })
                        .and_then(|n| field(*n, &["type"]))
                });
                let bool_field =
                    type_node.is_some_and(|n| exposed_boolean(n, &all, source, MAX_FLOW_DEPTH));
                if bool_field {
                    mechanism = Some(FIELD_ASSIGNMENT);
                }
            }
        }
        if capture == "unstructured_sink" {
            let arguments = args(node);
            let dictionary_update = report.language == "python"
                && receiver(node).is_some()
                && method(node, source) == "update";
            let bulk = callee(node).is_some_and(|n| {
                text(n, source) == "Object.assign" && resolve("Object", Span::of(n)).is_none()
            }) || dictionary_update
                || report.language == "python"
                    && method(node, source) == "setattr"
                    && resolve("setattr", Span::of(node)).is_none()
                || arguments.iter().any(|a| a.kind() == "dictionary_splat");
            if bulk {
                let sources: Vec<_> = if dictionary_update
                    || arguments.iter().any(|a| a.kind() == "dictionary_splat")
                {
                    arguments.clone()
                } else {
                    arguments.iter().skip(1).copied().collect()
                };
                for input in sources {
                    let resolved = value(input, &all, source, &resolve).unwrap_or(input);
                    let shaped = has_input_flow(resolved, &all, source, &resolve, MAX_FLOW_DEPTH)
                        || input.kind() == "identifier"
                            && resolve(text(input, source), Span::of(input)).is_some();
                    let input_binding = resolve(base(input, source), Span::of(input));
                    let invalidated = all.iter().any(|n| {
                        n.start_byte() > resolved.end_byte()
                            && n.end_byte() < node.start_byte()
                            && assignment(*n)
                            && left(*n).is_some_and(|l| {
                                resolve(base(l, source), Span::of(l)) == input_binding
                                    && input_binding.is_some()
                            })
                    });
                    let validated =
                        validated_result(resolved, &all, source, &resolve) && !invalidated;
                    if shaped && !validated {
                        mechanism = Some(DECODED_SINK);
                        origins.push(Span::of(resolved));
                    }
                }
                if method(node, source) == "setattr"
                    && arguments.get(1).is_some_and(|n| n.kind() == "identifier")
                {
                    mechanism = Some(DECODED_SINK);
                }
            }
            if assignment(node) {
                if let (Some(target), Some(input)) = (left(node), right(node)) {
                    if matches!(
                        target.kind(),
                        "subscript_expression" | "subscript" | "index_expression"
                    ) {
                        let index = field(target, &["index", "subscript"]);
                        if index.is_some_and(|n| n.kind() == "identifier")
                            && matches!(
                                input.kind(),
                                "subscript_expression" | "subscript" | "index_expression"
                            )
                        {
                            let receiver = field(input, &["object", "value", "operand"])
                                .or_else(|| input.named_child(0));
                            if receiver
                                .and_then(|r| value(r, &all, source, &resolve))
                                .is_some_and(|v| decoder(v, source, &resolve))
                            {
                                let allowed = ancestor(node, &["if_statement"])
                                    .and_then(|n| field(n, &["condition"]))
                                    .is_some_and(|condition| {
                                        let changed = all.iter().any(|n| {
                                            n.start_byte() > condition.end_byte()
                                                && n.end_byte() < node.start_byte()
                                                && (matches!(
                                                    n.kind(),
                                                    "await_expression" | "await"
                                                ) || assignment(*n)
                                                    && left(*n).is_some_and(|l| {
                                                        Some(text(l, source))
                                                            == index.map(|i| text(i, source))
                                                    }))
                                        });
                                        !changed
                                            && index.is_some_and(|index| {
                                                allowed_key(condition, index, source)
                                            })
                                    });
                                if !allowed {
                                    mechanism = Some(PROTOTYPE_KEY);
                                }
                            }
                        }
                    }
                    if report.language == "python"
                        && target.kind() == "attribute"
                        && calls(input)
                        && method(input, source) == "get"
                    {
                        let member = field(target, &["attribute"]).map(|n| text(n, source));
                        let bool_default = all.iter().filter(|n| assignment(**n)).any(|n| {
                            left(*n).is_some_and(|l| {
                                field(l, &["attribute"]).map(|n| text(n, source)) == member
                            }) && right(*n).is_some_and(|v| matches!(v.kind(), "true" | "false"))
                        });
                        if bool_default {
                            mechanism = Some(FIELD_ASSIGNMENT);
                        }
                    }
                    if report.language == "rust"
                        && ancestor(node, &["match_arm"]).is_some()
                        && input.kind() == "binary_expression"
                        && text(input, source).contains("==")
                    {
                        let bool_default = all.iter().filter(|n| assignment(**n)).any(|n| {
                            left(*n).is_some_and(|l| text(l, source) == text(target, source))
                                && right(*n).is_some_and(|v| v.kind() == "boolean_literal")
                        });
                        if bool_default {
                            mechanism = Some(FIELD_ASSIGNMENT);
                        }
                    }
                }
            }
            if matches!(node.kind(), "member_expression")
                && field(node, &["property"])
                    .is_some_and(|n| matches!(text(n, source), "constructor" | "__proto__"))
            {
                if let Some(decoded) = field(node, &["object"])
                    .and_then(|r| value(r, &all, source, &resolve))
                    .filter(|n| decoder(*n, source, &resolve))
                {
                    if args(decoded)
                        .get(1)
                        .is_none_or(|reviver| !filters_keys(*reviver, source))
                    {
                        mechanism = Some(PROTOTYPE_KEY);
                        origins.push(Span::of(decoded));
                    }
                }
            }
        }
        if capture == "unstructured_length" && report.language == "rust" {
            let index = field(node, &["index"]).or_else(|| node.named_child(1));
            if let Some(index) = index {
                let reads_prefix = identifiers(index).iter().any(|id| {
                    value(*id, &all, source, &resolve)
                        .is_some_and(|v| nodes(v).iter().any(|n| n.kind() == "index_expression"))
                });
                if reads_prefix {
                    mechanism = Some(UNTRUSTED_LENGTH);
                    origins.extend(
                        identifiers(index)
                            .iter()
                            .filter_map(|id| value(*id, &all, source, &resolve))
                            .filter(|v| nodes(*v).iter().any(|n| n.kind() == "index_expression"))
                            .map(Span::of),
                    );
                }
            }
        }
        if mechanism.is_none()
            && capture == "unstructured_decode"
            && (decoder(node, source, &resolve)
                || node.kind() == "attribute_item"
                    && text(node, source).starts_with("#[serde(")
                    && (text(node, source).contains("untagged")
                        || text(node, source).contains("flatten")))
        {
            crate::unstructured::usage(report, Span::of(node), DEFECT, DECODE_BOUNDARY, "A decode boundary introduces a runtime shape. This usage marker does not establish an unsafe loader, unchecked sink, or contract violation.");
        }
        if let Some(mechanism) = mechanism {
            add(report, Span::of(node), DEFECT, mechanism, "Decoded or unconstrained fields reach a structural operation without an established field or length contract. This is a local risk candidate, not proof of untrusted input or exploitation.", &origins);
            if calls(node) {
                let target = if method(node, source) == "update" {
                    receiver(node)
                } else {
                    args(node).first().copied()
                };
                if let Some(target) = target {
                    if let Some(bound) = resolve(base(target, source), Span::of(target))
                        .filter(|b| owner(report, node).is_some_and(|o| !o.contains(*b)))
                    {
                        crate::shared::emit(report, "cross-boundary-state-exposure", "shared-storage", vec![crate::shared::Event { kind: "shared-storage".into(), range: Span::of(node), resource: Some(format!("{}:{}", bound.start, bound.end)), owner: owner(report, node) }], false, crate::shared::SiteCounts::default(), "A structural update reaches an outer storage binding accessible beyond this invocation. This marks sharing, not concurrent execution.");
                    }
                }
            }
        }
    }
}

fn filters_keys(callback: Node<'_>, source: &str) -> bool {
    let Some(parameters) = field(callback, &["parameters"]) else {
        return false;
    };
    let Some(key) = parameters.named_child(0) else {
        return false;
    };
    nodes(callback).iter().any(|node| {
        if !matches!(node.kind(), "if_statement" | "ternary_expression") {
            return false;
        }
        let Some(condition) = field(*node, &["condition"]) else {
            return false;
        };
        let Some(branch) = field(*node, &["consequence"]) else {
            return false;
        };
        identifiers(condition)
            .iter()
            .any(|n| text(*n, source) == text(key, source))
            && nodes(branch)
                .iter()
                .any(|n| text(*n, source) == "undefined")
    })
}

fn validated_result(
    node: Node<'_>,
    all: &[Node<'_>],
    source: &str,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
) -> bool {
    if !calls(node) { return false; }
    let validation = if matches!(method(node, source), "model_dump" | "dict") {
        receiver(node).and_then(|n| value(n, all, source, resolve)).unwrap_or(node)
    } else { node };
    if matches!(method(validation, source), "model_validate" | "parse_obj") {
        let Some(model) = receiver(validation) else { return false; };
        let imported = all.iter().any(|n| n.kind() == "import_from_statement" && field(*n, &["module_name"]).is_some_and(|m| text(m, source) == "pydantic") && identifiers(*n).iter().any(|i| text(*i, source) == "BaseModel"));
        return imported && all.iter().any(|n| n.kind() == "class_definition" && field(*n, &["name"]).is_some_and(|name| text(name, source) == text(model, source)) && field(*n, &["superclasses"]).is_some_and(|bases| identifiers(bases).iter().any(|base| text(*base, source) == "BaseModel" && resolve("BaseModel", Span::of(*base)).is_none())) && !(text(*n, source).contains("extra") && (text(*n, source).contains("'allow'") || text(*n, source).contains("\"allow\""))));
    }
    if method(node, source) != "parse" { return false; }
    let Some(schema) = receiver(node).and_then(|n| value(n, all, source, resolve)) else {
        return false;
    };
    if !calls(schema) || !matches!(method(schema, source), "object" | "strictObject") {
        return false;
    }
    let Some(library) = receiver(schema) else {
        return false;
    };
    if resolve(text(library, source), Span::of(library)).is_some() { return false; }
    all.iter().any(|n| {
        n.kind() == "import_statement"
            && field(*n, &["source"])
                .is_some_and(|s| text(s, source).trim_matches(['\'', '"']) == "zod")
            && identifiers(*n)
                .iter()
                .any(|i| text(*i, source) == text(library, source))
    })
}

fn has_input_flow(
    node: Node<'_>,
    all: &[Node<'_>],
    source: &str,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
    depth: usize,
) -> bool {
    if depth == 0 || validated_result(node, all, source, resolve) {
        return false;
    }
    if decoder(node, source, resolve)
        || matches!(node.kind(), "object" | "dictionary" | "dictionary_splat")
    {
        return true;
    }
    if node.kind() == "identifier" {
        return value(node, all, source, resolve)
            .filter(|v| v.id() != node.id())
            .is_some_and(|v| has_input_flow(v, all, source, resolve, depth - 1));
    }
    args(node).iter().any(|n| has_input_flow(*n, all, source, resolve, depth - 1)) || receiver(node).is_some_and(|n| has_input_flow(n, all, source, resolve, depth - 1))
}

fn exposed_boolean(ty: Node<'_>, all: &[Node<'_>], source: &str, depth: usize) -> bool {
    if depth == 0 {
        return false;
    }
    if text(ty, source) == "bool" {
        return true;
    }
    let structure = if ty.kind() == "struct_type" {
        Some(ty)
    } else {
        all.iter()
            .find(|n| {
                n.kind() == "type_spec"
                    && field(**n, &["name"])
                        .is_some_and(|name| text(name, source) == text(ty, source))
            })
            .and_then(|n| field(*n, &["type"]))
    };
    structure.is_some_and(|structure| {
        nodes(structure).iter().any(|n| {
            n.kind() == "field_declaration"
                && !text(*n, source).contains("json:\"-\"")
                && field(*n, &["type"])
                    .is_some_and(|field_type| exposed_boolean(field_type, all, source, depth - 1))
        })
    })
}

fn allowed_key(condition: Node<'_>, key: Node<'_>, source: &str) -> bool {
    if condition.kind() == "parenthesized_expression" {
        return condition
            .named_child(0)
            .is_some_and(|n| allowed_key(n, key, source));
    }
    let (Some(left), Some(right)) = (field(condition, &["left"]), field(condition, &["right"]))
    else {
        return false;
    };
    let operator = source[left.end_byte()..right.start_byte()].trim();
    if operator == "&&" {
        return allowed_key(left, key, source) || allowed_key(right, key, source);
    }
    matches!(operator, "==" | "===")
        && text(left, source) == text(key, source)
        && matches!(right.kind(), "string" | "string_literal")
        && !matches!(
            text(right, source).trim_matches(['\'', '"']),
            "__proto__" | "constructor"
        )
}
