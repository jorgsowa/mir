===description===
reports scalar passed as nullable docblock intersection
===file===
<?php
interface Iterator {}
interface Countable {}

/** @param Iterator&Countable|null $x */
function f($x): void { $_ = $x; }

function test(): void {
    f("hello");
//    ^^^^^^^ InvalidArgument: Argument $x of f() expects 'Iterator&Countable|null', got '"hello"'
}
===expect===
