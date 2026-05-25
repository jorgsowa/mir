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
    }
}
===expect===
RedundantCondition
===ignore===
TODO
