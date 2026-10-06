===description===
Abstract extends non abstract with method
===file===
<?php
class A {
    public function foo() : void {}
}

abstract class B extends A {
    abstract public function foo() : void;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foo() signature mismatch: cannot make non-abstract method A::foo() abstract
}
