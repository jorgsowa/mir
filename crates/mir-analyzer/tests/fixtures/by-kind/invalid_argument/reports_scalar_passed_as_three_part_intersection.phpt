===description===
reports scalar passed as three part intersection
===file===
<?php
interface Iterator {}
interface Countable {}
interface Stringable {}

function f(Iterator&Countable&Stringable $x): void { $_ = $x; }

function test(): void {
    f("hello");
//    ^^^^^^^ InvalidArgument: Argument $x of f() expects 'Iterator&Countable&Stringable', got '"hello"'
}
===expect===
