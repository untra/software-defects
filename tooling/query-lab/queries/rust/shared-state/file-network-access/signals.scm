(call_expression
  function: [
    (identifier) @callee
    (field_expression field: (field_identifier) @callee)
    (scoped_identifier name: (identifier) @callee)
  ]
  (#any-of? @callee "exists" "write" "write_all" "create_new" "connect"))
