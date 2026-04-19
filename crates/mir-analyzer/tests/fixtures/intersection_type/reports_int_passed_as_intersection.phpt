===source===
<?php
interface Iterator {}
interface Countable {}

function f(Iterator&Countable $x): void { $_ = $x; }

function test(): void {
    f(42);
}
===expect===
InvalidArgument: 42
