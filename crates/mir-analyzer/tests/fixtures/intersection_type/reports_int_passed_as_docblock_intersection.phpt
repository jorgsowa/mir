===source===
<?php
interface Iterator {}
interface Countable {}

/** @param Iterator&Countable $x */
function f($x): void { $_ = $x; }

function test(): void {
    f(42);
}
===expect===
InvalidArgument: 42
