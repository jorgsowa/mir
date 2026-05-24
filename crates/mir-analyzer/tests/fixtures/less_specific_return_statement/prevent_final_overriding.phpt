===description===
preventFinalOverriding
===file===
<?php
/**
 * @psalm-consistent-constructor
 */
class A {
    /** @return static */
    public static function getInstance() {
        return new static();
    }
}

class AChild extends A {
    public static function getInstance() {
        return new AChild();
    }
}

class AGrandChild extends AChild {
    public function foo() : void {}
}

AGrandChild::getInstance()->foo();
===expect===
LessSpecificReturnStatement
===ignore===
TODO
