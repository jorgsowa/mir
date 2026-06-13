===description===
Possible raw object iteration
===config===
suppress=MissingPropertyType,UnusedParam
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
PossiblyRawObjectIteration@20:16-20:20: Cannot iterate over possibly non-iterable object 'list<A>|B'
