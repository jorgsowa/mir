===description===
InvalidPropertyFetch does NOT fire after instanceof narrows a union to a class type.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    public string $name = "x";
}

/** @var string|Foo $val */
$val = new Foo();
if ($val instanceof Foo) {
    $name = $val->name;
}
===expect===
