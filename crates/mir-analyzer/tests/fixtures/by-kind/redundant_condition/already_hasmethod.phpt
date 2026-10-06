===description===
Already hasmethod
===file===
<?php
class A {
    public function foo() : void {}
}

function foo(A $a) : void {
    if (method_exists($a, "foo")) {
        $object->foo();
//      ^^^^^^^^^^^^^^ MixedMethodCall: Method foo() called on mixed type
//      ^^^^^^^ UndefinedVariable: Variable $object is not defined
    }
}
