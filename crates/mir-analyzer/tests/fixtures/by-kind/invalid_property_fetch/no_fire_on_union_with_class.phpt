===description===
InvalidPropertyFetch does NOT fire when the union contains a class type alongside a scalar.
Note: there is no PossiblyInvalidPropertyFetch issue kind, so string|Foo produces no diagnostic.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    public string $name = "x";
}

/** @var string|Foo $val */
$val = new Foo();
$name = $val->name;
===expect===
