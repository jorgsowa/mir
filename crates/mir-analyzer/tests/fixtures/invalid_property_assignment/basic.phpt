===file===
<?php
class Foo {
    public string $name;
}

$f = new Foo();
$f->name = 42;
===expect===
InvalidPropertyAssignment: Property $name expects 'string', cannot assign '42'
