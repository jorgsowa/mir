===description===
An object-typed variable can never be === to a string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}

function test(Foo $obj): void {
    if ($obj === "foo") {}
//      ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'Foo' and '"foo"' is always false — these types can never be identical
    if ($obj === 42) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'Foo' and '42' is always false — these types can never be identical
}
===expect===
