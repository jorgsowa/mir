===description===
An object-typed variable can never be === to a string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}

function test(Foo $obj): void {
    if ($obj === "foo") {}
    if ($obj === 42) {}
}
===expect===
ImpossibleIdenticalComparison@5:8-5:22: '===' between 'Foo' and '"foo"' is always false — these types can never be identical
ImpossibleIdenticalComparison@6:8-6:19: '===' between 'Foo' and '42' is always false — these types can never be identical
