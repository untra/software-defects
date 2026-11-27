(parameter) @unstructured_parameter
(call_expression function: [(identifier) (field_expression field: (field_identifier)) (scoped_identifier name: (identifier))] @_callee (#match? @_callee "(^|\\.|::)(repeat|with_capacity|reserve|resize|collect)$")) @unstructured_resource
(for_expression) @unstructured_resource
(macro_invocation macro: (identifier) @_callee (#eq? @_callee "vec")) @unstructured_resource
