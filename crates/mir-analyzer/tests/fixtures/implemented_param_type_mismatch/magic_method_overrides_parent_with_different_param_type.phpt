===description===
Magic method overrides parent with different param type
===file===
<?php
class C {}
class D extends C {}

class A {
    public function foo(string $s) : C {
        return new C;
    }
}

/** @method D foo(int $s) */
class B extends A {}
===expect===
ImplementedParamTypeMismatch
===ignore===
TODO
