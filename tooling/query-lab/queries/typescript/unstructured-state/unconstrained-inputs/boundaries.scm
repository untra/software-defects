(formal_parameters (_) @unstructured_parameter)
(call_expression function: [(identifier) (member_expression property: (property_identifier))] @_callee (#match? @_callee "(^|\\.)(repeat|map|flatMap|slice|from)$")) @unstructured_resource
(for_in_statement) @unstructured_resource
(new_expression constructor: (identifier) @_callee (#any-of? @_callee "Array" "Uint8Array" "Int32Array" "Float64Array")) @unstructured_resource
(for_statement) @unstructured_resource
