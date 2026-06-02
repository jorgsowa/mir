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
PossiblyNullMethodCall@12:1-12:12: Cannot call method doBaz() on possibly null value
