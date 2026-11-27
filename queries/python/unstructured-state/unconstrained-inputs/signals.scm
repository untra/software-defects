(parameters (_) @unstructured_parameter)
(call function: [(identifier) (attribute attribute: (identifier))] @_callee (#match? @_callee "(^|\\.)(list|tuple|bytearray|bytes|range)$")) @unstructured_resource
(for_statement) @unstructured_resource
(binary_operator left: [(string) (list)] operator: "*" right: (_)) @unstructured_resource
