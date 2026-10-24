(call_expression
  function: [
    (identifier) @callee
    (field_expression field: (field_identifier) @callee)
    (scoped_identifier name: (identifier) @callee)
  ]
  (#any-of? @callee "get_or_init" "insert" "remove"))
