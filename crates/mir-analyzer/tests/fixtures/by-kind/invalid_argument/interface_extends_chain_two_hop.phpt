===description===
FN: an ancestor declared two `@extends`/`@implements` hops away (a class
implements a generic interface, which itself `@extends` a further generic
interface) was invisible to argument-compatibility checking. `InterfaceDef`
never stored its own `@extends Base<T>` type args, so the chain walk that
resolves what a concrete class supplies for a distant ancestor's template
params stopped after the first hop and fell back to unconditionally
accepting the argument.
===config===
suppress=MissingPropertyType,UnusedParam
===file===
<?php
/** @template-covariant E */
interface GrandCollection {}

/**
 * @template-covariant T
 * @extends GrandCollection<T>
 */
interface Collection extends GrandCollection {}

class Animal {}
class Unrelated {}

/**
 * @template T
 * @implements Collection<T>
 */
class TypedList implements Collection {
    /** @param T $item */
    public function __construct(private $item) {}
}

/** @param GrandCollection<Animal> $c */
function needsAnimals(GrandCollection $c): void {}

needsAnimals(new TypedList(new Unrelated()));
===expect===
InvalidArgument@26:13-26:43: Argument $c of needsAnimals() expects 'GrandCollection<Animal>', got 'TypedList<Unrelated>'
