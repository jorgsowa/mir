===description===
Basic
===file===
<?php
class Foo {
    public string $name;
}

$f = new Foo();
/** @mir-check $f is Foo */
$f->name = 42;
===expect===
MissingConstructor@2:0-2:11: Class Foo has uninitialized properties but no constructor
InvalidPropertyAssignment@8:0-8:13: Property $name expects 'string', cannot assign '42'
