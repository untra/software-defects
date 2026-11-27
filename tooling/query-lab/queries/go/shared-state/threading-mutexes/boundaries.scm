(call_expression
  function: [ (identifier) @mutex_method (selector_expression field: (field_identifier) @mutex_method) ]
  (#any-of? @mutex_method "Lock" "Unlock" "RLock" "RUnlock" "TryLock" "TryRLock")) @mutex_call
