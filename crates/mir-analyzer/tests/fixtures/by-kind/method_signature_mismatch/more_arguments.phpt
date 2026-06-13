===description===
More arguments
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {

    }
}

class B extends A {
    public function fooFoo(int $a, bool $b, array $c): void {

    }
}
===expect===
MethodSignatureMismatch@9:4-9:61: Method B::foofoo() signature mismatch: overriding method requires 3 argument(s) but parent requires 2
