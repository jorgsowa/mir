===description===
Fewer arguments
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {

    }
}

class B extends A {
    public function fooFoo(int $a): void {

    }
}
===expect===
MethodSignatureMismatch@9:4-9:42: Method B::foofoo() signature mismatch: method has fewer parameters (1) than parent A::foofoo() (2)
