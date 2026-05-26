===description===
reports string passed as intersection
===file===
<?php
interface Iterator {}
interface Countable {}

function f(Iterator&Countable $x): void { $_ = $x; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument@8:7: Argument $x of f() expects 'Iterator&Countable', got '"hello"'
