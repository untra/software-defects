(call_expression
  function: [ (identifier) @mutex_method (member_expression property: (property_identifier) @mutex_method) ]
  (#any-of? @mutex_method "acquire" "release" "request" "runExclusive")) @mutex_call
