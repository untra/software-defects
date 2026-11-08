(call_expression
  function: [
    (identifier) @callee
    (selector_expression field: (field_identifier) @callee)
  ]
  (#any-of? @callee "Lock" "Unlock" "Add" "Do" "append"))
