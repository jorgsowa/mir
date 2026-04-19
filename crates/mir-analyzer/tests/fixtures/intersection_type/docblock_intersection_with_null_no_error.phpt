===source===
<?php
interface Iterator {}
interface Countable {}

/** @param Iterator&Countable|null $x */
function f($x): void { $_ = $x; }

function test(): void {
    f(null);
}
===expect===
