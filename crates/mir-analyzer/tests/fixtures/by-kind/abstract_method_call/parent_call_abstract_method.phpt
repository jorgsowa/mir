===description===
AbstractMethodCall fires when calling an abstract method via parent::. The parent's abstract method has no body, so any call to it is a fatal runtime error.
===file===
<?php
abstract class Base {
    abstract public function foo(): void;
}
class Child extends Base {
    public function foo(): void {
        parent::foo();
    }
}
===expect===
AbstractMethodCall@7:8-7:21: Cannot call abstract method Base::foo()
