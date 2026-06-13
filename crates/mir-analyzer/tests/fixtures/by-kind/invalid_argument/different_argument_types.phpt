===description===
Different argument types
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {

    }
}

class B extends A {
    public function fooFoo(int $a, int $b): void {

    }
}
===expect===
MethodSignatureMismatch@9:4-9:50: Method B::foofoo() signature mismatch: parameter $b type 'int' is narrower than parent type 'bool'
