===description===
`self` from a grandparent stays the grandparent even when the method is called on a deeper subclass.
===config===
suppress=UnusedMethod
===file===
<?php
class A {
    public function me(): self { return $this; }
}
class B extends A {}
class C extends B {
    public function onlyC(): void {}
}

$c = new C();
/** @mir-check $c->me() is A */
$c->me()->onlyC();
===expect===
UndefinedMethod@12:0-12:17: Method A::onlyC() does not exist
