((call_expression function: (field_expression value: (_) @receiver field: (field_identifier) @callee))
 (#any-of? @callee "push" "pop" "insert" "remove" "swap_remove" "retain" "sort" "sort_by" "reverse" "truncate" "clear" "dedup" "push_str" "push_back" "push_front" "pop_back" "pop_front" "drain" "reserve" "or_insert" "and_modify"))
((call_expression function: (field_expression value: (_) @receiver field: (field_identifier) @access)) (#eq? @access "entry"))
