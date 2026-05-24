===description===
basic
===file===
<?php
class Foo {
    public string $name;
}

$f = new Foo();
/** @mir-check $f is Foo */
$f->name = 42;
===expect===
InvalidPropertyAssignment@8:1: Property $name expects 'string', cannot assign '42'
