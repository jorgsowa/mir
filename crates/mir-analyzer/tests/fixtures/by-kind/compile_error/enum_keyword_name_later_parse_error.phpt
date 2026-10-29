===description===
Only the parser's errors inside an `enum self` / `enum parent` declaration are its recovery; a parse error after it is still reported.
===file===
<?php
enum Self {
//   ^^^^ CompileError: Cannot use "Self" as an enum name as it is reserved [reserved-enum-name]
    case A;

    public function brace(): string {
        return '}';
    }
}

function f(): void {
    $x = ;
//       ^ ParseError: expected expression, found ';' [expected]
}
