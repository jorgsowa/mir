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
}
===expect===
InvalidArgument@9:6-9:13: Argument $x of f() expects 'Foo&Bar', got '"hello"'
