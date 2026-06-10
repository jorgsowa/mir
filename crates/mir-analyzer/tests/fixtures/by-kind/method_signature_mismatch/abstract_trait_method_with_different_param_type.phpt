===description===
Abstract trait method with different param type
===ignore===
TODO
===file===
<?php
class A {}
class B {}

trait T {
    abstract public function foo(A $a) : void;
}

class C {
    use T;

    public function foo(B $a) : void {}
}
===expect===
