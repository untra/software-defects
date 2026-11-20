(type_cast_expression) @unstructured_assertion
(call_expression function: [(identifier) (field_expression field: (field_identifier)) (scoped_identifier name: (identifier))] @_callee (#match? @_callee "(^|\\.|::)(downcast|downcast_ref|downcast_mut)$")) @unstructured_assertion
(call_expression function: (generic_function function: (field_expression field: (field_identifier) @_callee)) (#any-of? @_callee "downcast" "downcast_ref" "downcast_mut")) @unstructured_assertion
