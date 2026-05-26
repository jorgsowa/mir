===description===
Trait return type mismatch
===file===
<?php
class A {
    public function foo() : void {}
}

trait T {
    abstract public function foo() : string;
}

class B extends A {
    use T;
}
===expect===
MethodSignatureMismatch
===ignore===
TODO
