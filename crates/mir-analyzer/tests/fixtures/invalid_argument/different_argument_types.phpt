===description===
differentArgumentTypes
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
Argument 2 of B::fooFoo has wrong type \
===ignore===
TODO
