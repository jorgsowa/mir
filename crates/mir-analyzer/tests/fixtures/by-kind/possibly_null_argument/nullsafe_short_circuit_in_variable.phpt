===description===
Nullsafe short circuit in variable
===file===
<?php
interface Bar {
    public function doBaz(): void;
}
interface Foo {
    public function getBar(): Bar;
}
function fooOrNull(): ?Foo {
    return null;
}
$a = fooOrNull()?->getBar();
$a->doBaz();
===expect===
PossiblyNullMethodCall@12:0-12:11: Cannot call method doBaz() on possibly null value
