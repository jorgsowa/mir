===file===
<?php
interface Iterator {}
interface Countable {}
interface Stringable {}

function f(Iterator&Countable&Stringable $x): void { $_ = $x; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument: Argument $x of f() expects 'Iterator&Countable&Stringable', got '"hello"'
