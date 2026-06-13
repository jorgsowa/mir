===description===
no type not reported
===config===
suppress=MissingPropertyType
===file===
<?php
class Foo {
    public $name;
}

$f = new Foo();
$f->name = 42;
===expect===
