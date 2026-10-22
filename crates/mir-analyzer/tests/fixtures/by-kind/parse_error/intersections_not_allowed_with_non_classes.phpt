===description===
Intersections not allowed with non classes
===file===
<?php
interface A {
}
function foo (A&string $test): A&string {
    return $test;
}
===expect===
ParseError@4:16-4:22: Parse error: Type string cannot be part of an intersection type [builtin-type-in-intersection]
ParseError@4:33-4:39: Parse error: Type string cannot be part of an intersection type [builtin-type-in-intersection]
