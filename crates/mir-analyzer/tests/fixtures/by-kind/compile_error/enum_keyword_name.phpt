===description===
`enum self` / `enum parent` declare an enum with a reserved name, not the constant `enum` followed by a keyword.
===file===
<?php
interface I {}

enum Self implements I {
//   ^^^^ CompileError: Cannot use "Self" as an enum name as it is reserved [reserved-enum-name]
    case A;
}
