===description===
nullsafeShortCircuitInVariable
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
PossiblyNullReference
===ignore===
TODO
