(call_expression
  function: [
    (identifier) @callee
    (member_expression property: (property_identifier) @callee)
  ]
  (#any-of? @callee "forEach" "splice" "setTimeout"))
