===description===
Basic
===file===
<?php
class Foo {
//<^^^^^^^^^^^ MissingConstructor: Class Foo has uninitialized properties but no constructor
    public string $name;
}

$f = new Foo();
/** @mir-check $f is Foo */
$f->name = 42;
//<^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $name expects 'string', cannot assign '42'
===expect===
