===description===
Abstract trait method with different param type
===config===
suppress=UnusedParam
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
