(call_expression
  function: [
    (identifier) @callee
    (member_expression property: (property_identifier) @callee)
  ]
  (#any-of? @callee "on" "off" "setInterval" "clearInterval" "open" "close" "set"))
