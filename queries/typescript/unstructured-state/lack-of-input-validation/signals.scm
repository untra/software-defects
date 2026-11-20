(formal_parameters (_) @unstructured_parameter)
(call_expression function: [(identifier) (member_expression property: (property_identifier))] @_callee (#match? @_callee "(^|\\.)(JSON\\.parse)$")) @unstructured_boundary
