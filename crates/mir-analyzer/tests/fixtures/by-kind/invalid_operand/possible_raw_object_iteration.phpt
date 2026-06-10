===description===
Possible raw object iteration
===file===
<?php
class A {
    /** @var ?string */
    public $foo;
}

class B extends A {}

function bar(A $a): void {}

function gen() : Generator {
    $arr = [];

    if (rand(0, 10) > 5) {
        $arr[] = new A;
    } else {
        $arr = new B;
    }

    yield from $arr;
}
===expect===
