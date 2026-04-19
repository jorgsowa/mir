===source===
<?php
interface Iterator {}
interface Countable {}

/** @param Iterator&Countable|null $x */
function f($x): void { $_ = $x; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument: Argument $x of f() expects 'Iterator&Countable|null', got '"hello"'
