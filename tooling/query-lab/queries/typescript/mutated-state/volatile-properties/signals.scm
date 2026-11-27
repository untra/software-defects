((call_expression function: (member_expression object: (_) @receiver property: (property_identifier) @callee))
 (#any-of? @callee "push" "pop" "shift" "unshift" "splice" "sort" "reverse" "fill" "copyWithin" "add" "delete" "clear" "set"))
(assignment_expression left: (member_expression object: (_) @receiver) @operation)
(unary_expression operator: "delete" argument: (member_expression object: (_) @receiver) @operation)
