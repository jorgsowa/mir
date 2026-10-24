===description===
Intersections not allowed with non classes
===file===
<?php
interface A {
}
function foo (A&string $test): A&string {
//              ^^^^^^ CompileError: Compile error: Type string cannot be part of an intersection type [builtin-type-in-intersection]
//                               ^^^^^^ CompileError: Compile error: Type string cannot be part of an intersection type [builtin-type-in-intersection]
    return $test;
}
