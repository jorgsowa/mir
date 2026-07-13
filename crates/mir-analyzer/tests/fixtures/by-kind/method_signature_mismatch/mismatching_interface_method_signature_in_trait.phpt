===description===
Mismatching interface method signature in trait
===config===
suppress=UnusedParam
===file===
<?php
interface A {
    public function fooFoo(int $a, int $b): void;
}

trait T {
    public function fooFoo(int $a): void {
    }
}

class B implements A {
    use T;
}
===expect===
MethodSignatureMismatch@7:4-7:42: Method B::foofoo() signature mismatch: method has fewer parameters (1) than parent A::foofoo() (2)
