; Keywords
["and" "or" "implies" "as" "is" "contains" "in" "mod" "div"] @keyword

; Functions
(identifier) @function (#match? @function "^(where|select|first|last|tail|exists|all|empty|count|skip|take|union|intersect|exclude|distinct|aggregate|combine)$")

; Operators
["+" "-" "*" "/" "=" "!=" "<" ">" "<=" ">=" "&" "|" "~"] @operator

; Literals
(string) @string
(number) @number
["true" "false"] @constant.builtin

; Comments
(comment) @comment

; Properties
(member_expression property: (identifier) @property)

; Variables
(variable) @variable

; Parentheses and brackets
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Delimiters
["." "," ";" ":"] @punctuation.delimiter
