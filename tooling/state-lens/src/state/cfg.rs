use super::*;

#[derive(Clone, Copy)]
struct Block(usize);

pub(super) struct Flow {
    current: Option<Block>,
    exit: Block,
    breaks: Vec<Block>,
}

impl<F: Fn(&str, Span) -> Option<Span>> Builder<'_, F> {
    fn block(&mut self) -> Block {
        let index = self.graph.blocks.len();
        self.graph.blocks.push(BasicBlock {
            id: format!("block:{index}"),
            operations: vec![],
        });
        Block(index)
    }
    fn connect(&mut self, from: Block, to: Block, branch: Branch, condition: Option<String>) {
        let evidence = condition
            .as_ref()
            .and_then(|id| self.graph.conditions.iter().find(|item| &item.id == id))
            .map(|item| item.evidence.clone())
            .unwrap_or_default();
        self.graph.control_flow.push(ControlFlowEdge {
            from: self.graph.blocks[from.0].id.clone(),
            to: self.graph.blocks[to.0].id.clone(),
            branch,
            condition,
            evidence,
        });
    }
    fn current(&mut self) -> Block {
        if let Some(block) = self.flow.as_ref().unwrap().current {
            return block;
        }
        let block = self.block();
        self.flow.as_mut().unwrap().current = Some(block);
        block
    }
    fn go(&mut self, block: Block) {
        self.flow.as_mut().unwrap().current = Some(block);
    }
    fn close(&mut self, target: Block) {
        if let Some(from) = self.flow.as_mut().unwrap().current.take() {
            self.connect(from, target, Branch::Unconditional, None);
        }
    }
    pub(super) fn start_flow(&mut self) {
        let entry = self.block();
        let exit = self.block();
        self.flow = Some(Flow {
            current: Some(entry),
            exit,
            breaks: vec![],
        });
    }
    pub(super) fn finish_flow(&mut self) {
        if let Some(flow) = &self.flow {
            let exit = flow.exit;
            self.close(exit);
        }
    }
    pub(super) fn attach(&mut self, id: OperationId) {
        if self.flow.is_none() {
            return;
        }
        let block = self.current();
        self.graph.operations[id].block_id = Some(self.graph.blocks[block.0].id.clone());
        self.graph.blocks[block.0]
            .operations
            .push(self.graph.operations[id].identity.clone());
    }
    fn condition(
        &mut self,
        node: Node<'_>,
        inputs: Vec<OperationId>,
        guards: &[OperationId],
        meaning: &str,
    ) -> String {
        let op = self.add(node, OperationKind::Guard, guards);
        for input in inputs {
            self.edge(input, op, Relationship::ValueDependency);
        }
        let id = format!("condition:{}", self.graph.conditions.len());
        self.graph.conditions.push(Condition {
            id: id.clone(),
            predicate: Predicate::Unknown {
                reason: format!("{meaning}: {}", &self.source[node.byte_range()]),
            },
            uncertainty: vec!["Predicate evaluation is not resolved.".into()],
            evidence: vec![format!("evidence:{op}")],
        });
        id
    }
    fn split(&mut self, condition: String) -> (Block, Block) {
        let from = self.current();
        let yes = self.block();
        let no = self.block();
        self.connect(from, yes, Branch::True, Some(condition.clone()));
        self.connect(from, no, Branch::False, Some(condition));
        (yes, no)
    }
    pub(super) fn flow_visit(
        &mut self,
        n: Node<'_>,
        defs: &mut Definitions,
        guards: &[OperationId],
    ) -> Option<Vec<OperationId>> {
        match n.kind() {
            "function_declaration"
            | "generator_function_declaration"
            | "function_expression"
            | "generator_function"
            | "arrow_function"
            | "method_definition" => {
                let outer = self.flow.take();
                self.start_flow();
                let mut local = Definitions::new();
                if let Some(params) = n
                    .child_by_field_name("parameters")
                    .or_else(|| n.child_by_field_name("parameter"))
                {
                    self.visit(params, &mut local, &[]);
                }
                if let Some(body) = n.child_by_field_name("body") {
                    let inputs = self.visit(body, &mut local, &[]);
                    if n.kind() == "arrow_function" && body.kind() != "statement_block" {
                        let ret = self.add(body, OperationKind::Return, &[]);
                        for input in inputs {
                            self.edge(input, ret, Relationship::ValueDependency);
                        }
                    }
                }
                self.finish_flow();
                self.flow = outer;
                Some(vec![])
            }
            "return_statement" | "throw_statement" => {
                let mut inputs = vec![];
                for child in children(n) {
                    inputs.extend(self.visit(child, defs, guards));
                }
                let kind = if n.kind() == "return_statement" {
                    OperationKind::Return
                } else {
                    OperationKind::ExceptionalExit
                };
                let op = self.add(n, kind, guards);
                for input in inputs {
                    self.edge(input, op, Relationship::ValueDependency);
                }
                let exit = self.flow.as_ref().unwrap().exit;
                self.close(exit);
                if kind == OperationKind::ExceptionalExit {
                    self.graph
                        .limitations
                        .push("Exception completion is unresolved.".into());
                }
                Some(vec![op])
            }
            "if_statement" | "ternary_expression" => {
                let condition = n.child_by_field_name("condition")?;
                let inputs = self.visit(condition, defs, guards);
                let condition = self.condition(condition, inputs, guards, "JavaScript truthiness");
                let mut nested = guards.to_vec();
                nested.push(self.graph.operations.len() - 1);
                let guards = nested.as_slice();
                let (yes, no) = self.split(condition);
                let mut yes_defs = defs.clone();
                let mut no_defs = defs.clone();
                self.go(yes);
                let mut result = n
                    .child_by_field_name("consequence")
                    .map(|body| self.visit(body, &mut yes_defs, guards))
                    .unwrap_or_default();
                let yes_end = self.flow.as_mut().unwrap().current.take();
                self.go(no);
                if let Some(body) = n.child_by_field_name("alternative") {
                    result.extend(self.visit(body, &mut no_defs, guards));
                }
                let no_end = self.flow.as_mut().unwrap().current.take();
                defs.clear();
                if yes_end.is_some() {
                    merge(defs, yes_defs);
                }
                if no_end.is_some() {
                    merge(defs, no_defs);
                }
                if yes_end.is_some() || no_end.is_some() {
                    let join = self.block();
                    for end in [yes_end, no_end].into_iter().flatten() {
                        self.connect(end, join, Branch::Unconditional, None);
                    }
                    self.go(join);
                }
                Some(result)
            }
            "binary_expression" => {
                let operator = n
                    .child_by_field_name("operator")
                    .map(|op| &self.source[op.byte_range()])?;
                if !matches!(operator, "&&" | "||" | "??") {
                    return None;
                }
                let left = n.child_by_field_name("left")?;
                let right = n.child_by_field_name("right")?;
                let mut result = self.visit(left, defs, guards);
                let condition = self.condition(
                    left,
                    result.clone(),
                    guards,
                    if operator == "??" {
                        "JavaScript nullishness"
                    } else {
                        "JavaScript truthiness"
                    },
                );
                let (yes, no) = self.split(condition);
                let (evaluate, skip) = if operator == "||" {
                    (no, yes)
                } else {
                    (yes, no)
                };
                let before = defs.clone();
                self.go(evaluate);
                result.extend(self.visit(right, defs, guards));
                let join = self.block();
                self.close(join);
                self.connect(skip, join, Branch::Unconditional, None);
                self.go(join);
                merge(defs, before);
                Some(result)
            }
            "switch_statement" => {
                let value = n.child_by_field_name("value")?;
                let inputs = self.visit(value, defs, guards);
                let body = n.child_by_field_name("body")?;
                let cases = children(body);
                let exit = self.block();
                let blocks: Vec<_> = cases.iter().map(|_| self.block()).collect();
                let mut default = exit;
                for (case, block) in cases.iter().zip(&blocks) {
                    if let Some(test) = case.child_by_field_name("value") {
                        let mut test_inputs = inputs.clone();
                        test_inputs.extend(self.visit(test, defs, guards));
                        let condition = self.condition(
                            test,
                            test_inputs,
                            guards,
                            "JavaScript strict equality with switch discriminant",
                        );
                        let from = self.current();
                        let next = self.block();
                        self.connect(from, *block, Branch::True, Some(condition.clone()));
                        self.connect(from, next, Branch::False, Some(condition));
                        self.go(next);
                    } else {
                        default = *block;
                    }
                }
                self.close(default);
                self.flow.as_mut().unwrap().breaks.push(exit);
                let mut joined = defs.clone();
                for (index, case) in cases.iter().enumerate() {
                    self.go(blocks[index]);
                    let mut local = joined.clone();
                    for child in children(*case) {
                        if Some(child) != case.child_by_field_name("value") {
                            self.visit(child, &mut local, guards);
                        }
                    }
                    merge(&mut joined, local);
                    self.close(blocks.get(index + 1).copied().unwrap_or(exit));
                }
                self.flow.as_mut().unwrap().breaks.pop();
                self.go(exit);
                *defs = joined;
                Some(vec![])
            }
            "break_statement"
                if n.named_child_count() == 0 && !self.flow.as_ref().unwrap().breaks.is_empty() =>
            {
                let target = *self.flow.as_ref().unwrap().breaks.last().unwrap();
                self.close(target);
                Some(vec![])
            }
            "for_statement" | "for_in_statement" | "while_statement" | "do_statement"
            | "try_statement" | "break_statement" | "continue_statement" | "labeled_statement" => {
                self.graph.limitations.push(
                    "Loops, labeled exits and exception completion have unresolved control flow."
                        .into(),
                );
                for child in children(n) {
                    self.visit(child, defs, guards);
                }
                Some(vec![])
            }
            _ => None,
        }
    }
}
fn merge(target: &mut Definitions, source: Definitions) {
    for (binding, values) in source {
        target.entry(binding).or_default().extend(values);
    }
}

impl StateGraph {
    pub(super) fn qualify_flow(&mut self, snapshot: &str) {
        let uri = self
            .operations
            .first()
            .map(|op| op.location.uri.as_str())
            .or_else(|| self.documents.first().map(|doc| doc.uri.as_str()))
            .unwrap_or_default();
        let prefix = format!("{snapshot}:{uri}");
        let mut mapping = BTreeMap::new();
        for op in &self.operations {
            mapping.insert(op.identity.clone(), format!("{prefix}:operation:{}", op.id));
            mapping.insert(
                format!("evidence:{}", op.id),
                format!("{prefix}:evidence:{}", op.id),
            );
            for evidence in &op.evidence {
                mapping.insert(evidence.clone(), format!("{prefix}:evidence:{}", op.id));
            }
        }
        for (index, block) in self.blocks.iter().enumerate() {
            mapping.insert(block.id.clone(), format!("{prefix}:block:{index}"));
        }
        for (index, condition) in self.conditions.iter().enumerate() {
            mapping.insert(condition.id.clone(), format!("{prefix}:condition:{index}"));
        }
        let remap = |id: &mut String| {
            if let Some(new) = mapping.get(id) {
                *id = new.clone();
            }
        };
        for block in &mut self.blocks {
            remap(&mut block.id);
            for op in &mut block.operations {
                remap(op);
            }
        }
        for op in &mut self.operations {
            if let Some(block) = &mut op.block_id {
                remap(block);
            }
        }
        for condition in &mut self.conditions {
            remap(&mut condition.id);
            for evidence in &mut condition.evidence {
                remap(evidence);
            }
        }
        for edge in &mut self.control_flow {
            remap(&mut edge.from);
            remap(&mut edge.to);
            if let Some(condition) = &mut edge.condition {
                remap(condition);
            }
            for evidence in &mut edge.evidence {
                remap(evidence);
            }
        }
    }
}
