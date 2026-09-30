===description===
Magic method overrides parent with different return type
===config===
suppress=UnusedParam
===file===
<?php
class C {}
class D {}

class A {
    public function foo(string $s) : C {
        return new C;
    }
}

/** @method D foo(string $s) */
class B extends A {}
//<^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foo() signature mismatch: return type 'D' is not a subtype of parent 'C'
===expect===
