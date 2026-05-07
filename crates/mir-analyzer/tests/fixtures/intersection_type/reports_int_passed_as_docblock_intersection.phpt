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
InvalidArgument@9:6: Argument $x of f() expects 'Iterator&Countable', got '42'
