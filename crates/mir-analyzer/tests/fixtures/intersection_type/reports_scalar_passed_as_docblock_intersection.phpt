===source===
<?php
interface Iterator {}
interface Countable {}

/** @param Iterator&Countable $x */
function f($x): void { $_ = $x; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument: "hello"
