===description===
abstractTraitMethodWithDifferentParamType
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
TraitMethodSignatureMismatch
===ignore===
TODO
