===description===
Raw object iteration
===file===
<?php
class A {
    /** @var ?string */
    public $foo;
}
function example() : Generator {
    $arr = new A;

    yield from $arr;
}
===expect===
RawObjectIteration@9:16-9:20: Cannot iterate over non-iterable object 'A'
