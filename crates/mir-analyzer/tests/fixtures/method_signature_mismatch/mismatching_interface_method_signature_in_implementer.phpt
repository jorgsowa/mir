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
MethodSignatureMismatch
===ignore===
TODO
