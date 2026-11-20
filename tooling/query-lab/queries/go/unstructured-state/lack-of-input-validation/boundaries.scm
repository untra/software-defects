(function_declaration parameters: (parameter_list [(parameter_declaration) (variadic_parameter_declaration)] @unstructured_parameter))
(method_declaration parameters: (parameter_list [(parameter_declaration) (variadic_parameter_declaration)] @unstructured_parameter))
(func_literal parameters: (parameter_list [(parameter_declaration) (variadic_parameter_declaration)] @unstructured_parameter))
(call_expression function: [(identifier) (selector_expression field: (field_identifier))] @_callee (#match? @_callee "(^|\\.)(Atoi|ParseInt|ParseFloat|Unmarshal|Decode)$")) @unstructured_boundary
