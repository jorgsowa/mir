===description===
A duplicate part among three (`Foo&Bar&Foo`) must drop only the repeated
`Foo`, printing `Foo&Bar` — distinct, non-adjacent parts are left in place.
===file===
<?php
interface Foo {}
interface Bar {}

/** @param Foo&Bar&Foo $x */
function f($x): void { $_ = $x; }

function test(): void {
    f("hello");
//    ^^^^^^^ InvalidArgument: Argument $x of f() expects 'Foo&Bar', got '"hello"'
}
===expect===
