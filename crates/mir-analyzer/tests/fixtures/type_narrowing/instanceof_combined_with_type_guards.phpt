===description===
instanceof narrowing combined with type guards
===file===
<?php
class Foo {
    public function foo() {}
}

class Bar {
    public function bar() {}
}

function testInstanceofWithTypeGuard($x) {
    if ($x instanceof Foo) {
        $x->foo();
    }
}

function testTypeGuardAfterInstanceof($x, $y) {
    if ($x instanceof Foo) {
        $x->foo();
    }

    if (is_string($y)) {
        strlen($y);
    }
}

function testNegatedInstanceof($x) {
    if (!($x instanceof Foo)) {
        return null;
    }
    $x->foo();
}

function mixedNarrowing($value) {
    if (is_object($value) && $value instanceof Foo) {
        $value->foo();
    } elseif (is_array($value)) {
        count($value);
    }
}
===expect===
