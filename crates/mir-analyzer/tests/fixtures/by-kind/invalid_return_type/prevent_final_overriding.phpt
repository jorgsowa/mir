===description===
Prevent final overriding
===file===
<?php
/**
 * @consistent-constructor
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
UndefinedMethod@22:1-22:34: Method AChild::foo() does not exist
