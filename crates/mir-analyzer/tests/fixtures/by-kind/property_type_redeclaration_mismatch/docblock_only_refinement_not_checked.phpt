===description===
Child redeclares parent property with the same native type hint but a more specific
`@var` docblock — PHP's redeclaration-invariance rule only checks the native hint, so
this is valid and must not raise PropertyTypeRedeclarationMismatch.
===config===
suppress=MissingPropertyType
===file===
<?php
class Animal {}
class Dog extends Animal {}

class A {
    /** @var array<int, Animal> */
    public array $items = [];
}

class B extends A {
    /** @var array<int, Dog> */
    public array $items = [];
}

class C {
    /** @var positive-int */
    public int $count = 1;
}

class D extends C {
    /** @var int */
    public int $count = 1;
}
===expect===
