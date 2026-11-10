===description===
Child redeclares parent property with the same native type hint but a more specific
`@var` docblock — PHP's redeclaration-invariance rule only checks the native hint, so
this is valid and must not raise PropertyTypeRedeclarationMismatch.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ NonInvariantDocblockPropertyType: Docblock type 'array<int, Dog>' of B::$items is not invariant with 'array<int, Animal>' in class A
}

class C {
    /** @var positive-int */
    public int $count = 1;
}

class D extends C {
    /** @var int */
    public int $count = 1;
//  ^^^^^^^^^^^^^^^^^^^^^^ NonInvariantDocblockPropertyType: Docblock type 'int' of D::$count is not invariant with 'positive-int' in class C
}
