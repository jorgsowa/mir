===description===
FP: a `@template T of Collection<Animal>` bound must accept a `TypedList<Dog>`
argument when `TypedList` is itself generic (`@template T`) and forwards its
own `T` to `Collection` via `@implements Collection<T>`, given `Collection`'s
`T` is `@template-covariant` and `Dog extends Animal`. Exercises
`is_subtype`'s cross-hierarchy variance path directly (not through function
argument checking).
===config===
suppress=ForbiddenCode,UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
/** @template-covariant T */
interface Collection {}

class Animal {}
class Dog extends Animal {}
class Unrelated {}

/**
 * @template T
 * @implements Collection<T>
 */
class TypedList implements Collection {
    /** @param T $item */
    public function __construct(private $item) {}
}

/**
 * @template T of Collection<Animal>
 * @param T $c
 */
function accept($c): void {}

accept(new TypedList(new Dog()));

/**
 * @template T of Collection<Animal>
 * @param T $c
 */
function accept_bad($c): void {}

accept_bad(new TypedList(new Unrelated()));
===expect===
InvalidTemplateParam@32:0-32:42: Template type 'T' inferred as 'TypedList<Unrelated>' does not satisfy bound 'Collection<Animal>'
