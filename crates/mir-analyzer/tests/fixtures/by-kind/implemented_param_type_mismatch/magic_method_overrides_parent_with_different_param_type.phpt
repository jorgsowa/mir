===description===
Magic method overrides parent with different param type
===config===
suppress=UnusedParam
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
MethodSignatureMismatch@12:0-12:20: Method B::foo() signature mismatch: parameter $s type 'int' is narrower than parent type 'string'
