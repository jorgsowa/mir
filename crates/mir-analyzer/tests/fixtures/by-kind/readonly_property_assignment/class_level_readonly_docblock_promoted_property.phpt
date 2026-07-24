===description===
A class-level `@readonly` docblock tag also reaches a promoted
constructor property — the promoted-property `is_readonly` computation
is a separate site from the plain-property one and needed the same fix.
===file===
<?php
/** @readonly */
class Foo {
    public function __construct(public string $name) {
    }
}

$foo = new Foo("a");
$foo->name = "b";
===expect===
ReadonlyPropertyAssignment@9:0-9:16: Cannot assign to readonly property Foo::$name outside of constructor
