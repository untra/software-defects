(call
  function: [ (identifier) @mutex_method (attribute attribute: (identifier) @mutex_method) ]
  (#any-of? @mutex_method "Lock" "RLock" "acquire" "release")) @mutex_call
