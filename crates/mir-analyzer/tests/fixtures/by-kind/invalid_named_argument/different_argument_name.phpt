===description===
Different argument name
===file===
<?php
class A {
    public function fooFoo(int $a): void {

    }
}

class B extends A {
    public function fooFoo(int $b): void {

    }
}
===expect===
ParamNameMismatch
===ignore===
TODO
