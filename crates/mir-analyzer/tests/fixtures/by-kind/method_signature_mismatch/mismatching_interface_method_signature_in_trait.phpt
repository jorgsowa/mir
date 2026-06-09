===description===
Mismatching interface method signature in trait
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
MethodSignatureMismatch@11:0-11:22: Method B::foofoo() signature mismatch: method has fewer parameters (1) than interface A::foofoo() (2)
