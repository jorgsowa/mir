===description===
Already hasmethod
===ignore===
TODO
===file===
<?php
class A {
    public function foo() : void {}
}

function foo(A $a) : void {
    if (method_exists($a, "foo")) {
        $object->foo();
    }
}
===expect===
MixedMethodCall@8:9-8:23: Method foo() called on mixed type
UndefinedVariable@8:9-8:16: Variable $object is not defined
