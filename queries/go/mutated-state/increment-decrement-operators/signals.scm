[(inc_statement) (dec_statement)] @update
(call_expression function: (identifier) @cast (#match? @cast "^(u?int(8|16|32|64)?|byte|rune)$")) @numeric_cast
(type_conversion_expression) @numeric
