===description===
mismatchingInterfaceMethodSignature
===file===
<?php
interface A {
    public function fooFoo(int $a): void;
}

class B implements A {
    public function fooFoo(string $a): void {

    }
}
===expect===
MethodSignatureMismatch
===ignore===
TODO
