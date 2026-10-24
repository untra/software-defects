(call_expression
  function: [
    (identifier) @callee
    (member_expression property: (property_identifier) @callee)
  ]
  (#any-of? @callee "existsSync" "writeFile" "open" "write" "fetch" "postCharge"))
