===description===
reports int passed as docblock intersection
===file===
<?php
interface Iterator {}
interface Countable {}

/** @param Iterator&Countable $x */
function f($x): void { $_ = $x; }

function test(): void {
    f(42);
}
===expect===
InvalidArgument: Argument $x of f() expects 'Iterator&Countable', got '42'
===ignore===
TODO
