(call_expression
  function: [
    (identifier) @callee
    (selector_expression field: (field_identifier) @callee)
  ]
  (#any-of? @callee "Record" "Report" "Allow"))
