===description===
mismatchingInterfaceMethodSignatureInTrait
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
MethodSignatureMismatch
===ignore===
TODO
