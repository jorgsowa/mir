===description===
@property docblock magic properties are not real PHP properties and establish
no visibility contract. A child class declaring a real property with the same
name and any visibility must NOT emit OverriddenPropertyAccess.
===config===
suppress=MissingPropertyType
===file===
<?php

/** @property string $foo */
class A {
    public function __get(string $key): mixed { return null; }
}

class B extends A {
    private string $foo = '';
}

class C extends A {
    protected string $foo = '';
}
===expect===
