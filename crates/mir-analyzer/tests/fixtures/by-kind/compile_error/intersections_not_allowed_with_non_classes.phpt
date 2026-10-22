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
CompileError@4:16-4:22: Compile error: Type string cannot be part of an intersection type [builtin-type-in-intersection]
CompileError@4:33-4:39: Compile error: Type string cannot be part of an intersection type [builtin-type-in-intersection]
