===description===
A bare subclass's own type arg (`DogList<Cat>`, `DogList` declaring no own
`@template`) must substitute into its ancestor's `@implements Collection<T>`
binding instead of leaking the raw unbound `T`, so passing it where a
`Collection<Dog>` is expected is correctly rejected as `Collection<Cat>`,
not vacuously accepted via an unresolved template.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
/** @template-covariant T */
interface Collection {}

/**
 * @template T
 * @implements Collection<T>
 */
class TypedList implements Collection {}

class DogList extends TypedList {}

class Dog {}
class Cat {}

/** @param Collection<Dog> $c */
function accept_dog_collection($c): void {}

/** @param DogList<Cat> $list */
function relay($list): void {
    accept_dog_collection($list);
}
===expect===
InvalidArgument@21:26-21:31: Argument $c of accept_dog_collection() expects 'Collection<Dog>', got 'DogList<Cat>'
