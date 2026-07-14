===description===
A redundant `Foo&Foo` intersection (two structurally identical parts) must
print as the deduplicated `Foo`, not the verbatim `Foo&Foo`.
===file===
<?php
interface Foo {}

/** @param Foo&Foo $x */
function f($x): void { $_ = $x; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument@8:6-8:13: Argument $x of f() expects 'Foo', got '"hello"'
