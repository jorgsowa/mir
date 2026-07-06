===description===
FP: a class that is ITSELF generic (`@template T`) and forwards its own T to a
`@template-covariant` interface (`@implements Collection<T>`) must satisfy the
interface parameterized with a supertype of its own concrete T — not just the
fixed-args case (`@extends Box<Cat>` with no class-level template of its own).
===config===
suppress=ForbiddenCode,UnusedVariable,MissingPropertyType
===file===
<?php
/** @template-covariant T */
interface Collection {}

class Animal {}
class Dog extends Animal {}

/**
 * @template T
 * @implements Collection<T>
 */
class TypedList implements Collection {
    /** @param T $item */
    public function __construct(private $item) {}
}

/** @param Collection<Animal> $c */
function needsAnimals(Collection $c): void { var_dump($c); }

/** @param Collection<Dog> $c */
function needsDogs(Collection $c): void { var_dump($c); }

function test(): void {
    $dogs = new TypedList(new Dog());
    needsAnimals($dogs);

    // Still a genuine violation: an Animal-typed list does not satisfy a
    // Dog-only collection, even through the covariant interface.
    $animals = new TypedList(new Animal());
    needsDogs($animals);
}
===expect===
InvalidArgument@30:14-30:22: Argument $c of needsDogs() expects 'Collection<Dog>', got 'TypedList<Animal>'
