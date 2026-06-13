===description===
ImpurePropertyAssignment does NOT fire when the property is on a locally created
object (not a parameter) inside a @pure function.
===file===
<?php
class Foo {
    public int $a = 0;
}

/** @pure */
function localOnly(int $n): Foo {
    $obj = new Foo();
    $obj->a = $n;
    return $obj;
}

===expect===
