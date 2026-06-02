===description===
Mismatching interface method signature in implementer
===file===
<?php
interface A {
    public function fooFoo(int $a, int $b): void;
}

trait T {
    public function fooFoo(int $a, int $b): void {
    }
}

class B implements A {
    use T;

    public function fooFoo(int $a): void {
    }
}
===expect===
MethodSignatureMismatch@14:4-14:42: Method B::foofoo() signature mismatch: method has fewer parameters (1) than parent T::foofoo() (2)
