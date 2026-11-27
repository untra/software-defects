(call_expression
  function: [ (identifier) @mutex_method (field_expression field: (field_identifier) @mutex_method) ]
  (#any-of? @mutex_method "lock" "try_lock" "read" "write" "try_read" "try_write")) @mutex_call
