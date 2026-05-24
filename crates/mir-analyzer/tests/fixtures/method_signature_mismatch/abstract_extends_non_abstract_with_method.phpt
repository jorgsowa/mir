===description===
abstractExtendsNonAbstractWithMethod
===file===
<?php
class A {
    public function foo() : void {}
}

abstract class B extends A {
    abstract public function foo() : void;
}
===expect===
MethodSignatureMismatch
===ignore===
TODO
