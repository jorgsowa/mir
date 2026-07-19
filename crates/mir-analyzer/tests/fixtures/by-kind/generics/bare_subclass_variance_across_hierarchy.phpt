===description===
A bare subclass that doesn't redeclare @template (`class DogBox extends Box
{}`) still satisfies an `@if-this-is` constraint on a covariant ancestor
interface through its inherited type arg, the same way a directly-generic
class already does. A genuine mismatch is still flagged.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingThrowsDocblock
===file===
<?php
class Animal {}
class Dog extends Animal {}
class Widget {}

/** @template-covariant T */
interface Collection {}

/**
 * @template T
 * @implements Collection<T>
 */
class Box implements Collection {
    /** @if-this-is Collection<Animal> */
    public function requiresAnimalCollection(): void {}
}

class DogBox extends Box {}

function testCovariantThroughAncestor(): void {
    /** @var DogBox<Dog> $box */
    $box = new DogBox();
    $box->requiresAnimalCollection();
}

function testMismatchStillFlagged(): void {
    /** @var DogBox<Widget> $box */
    $box = new DogBox();
    $box->requiresAnimalCollection();
}
===expect===
IfThisIsMismatch@29:4-29:36: Cannot call DogBox::requiresAnimalCollection() — @if-this-is requires $this to be 'Collection<Animal>', but it is 'DogBox<Widget>'
