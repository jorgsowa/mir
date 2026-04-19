===source===
<?php
interface Iterator {}
interface Countable {}

function f(Iterator&Countable $x): void { $_ = $x; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument: "hello"
