===description===
Raw object iteration
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var ?string */
    public $foo;
}
function example() : Generator {
    $arr = new A;

    yield from $arr;
//             ^^^^ RawObjectIteration: Cannot iterate over non-iterable object 'A'
}
===expect===
