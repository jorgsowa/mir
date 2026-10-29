===description===
Intersections not allowed with unions
===file===
<?php
interface A {
}
interface B {
}
interface C {
}
function foo (A&B|C $test): A&B|C {
//               ^ ParseError: expected '(' around an intersection type in a union, found '|' [expected]
//                             ^ ParseError: expected '(' around an intersection type in a union, found '|' [expected]
    return $test;
}
