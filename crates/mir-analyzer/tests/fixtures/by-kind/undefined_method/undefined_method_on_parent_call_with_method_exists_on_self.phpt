===description===
Undefined method on parent call with method exists on self
===file===
<?php
class A {}
class B extends A {
    public function foo(): string {
        return parent::foo();
    }
}
===expect===
UndefinedMethod
===ignore===
TODO
