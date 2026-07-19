===description===
`T|array<T>` (and `T|Closure(): T`) no longer let the container alternative
"explain away" an out-of-bound element without checking it — the container's
own element/return type is still bound and checked against `T`'s bound,
while a legitimately-in-bound element stays silent.
===config===
suppress=UnusedParam,MissingThrowsDocblock
===file===
<?php
class Animal {}
class Dog extends Animal {}
class NotAnimal {}

/**
 * @template T of Animal
 * @param T|array<T> $value
 */
function processArrayOrSingle($value): void {}

/** @param array<Dog> $dogs */
function test_array_of_bound_subtype_is_fine($dogs): void {
    processArrayOrSingle($dogs);
}

/** @param array<NotAnimal> $notAnimals */
function test_array_element_violating_bound_is_flagged($notAnimals): void {
    processArrayOrSingle($notAnimals);
}

/**
 * @template T of Animal
 * @param T|Closure(): T $value
 */
function processClosureOrSingle($value): void {}

/** @param Closure(): Dog $f */
function test_closure_returning_bound_subtype_is_fine($f): void {
    processClosureOrSingle($f);
}

/** @param Closure(): NotAnimal $f */
function test_closure_return_violating_bound_is_flagged($f): void {
    processClosureOrSingle($f);
}
===expect===
InvalidTemplateParam@19:4-19:37: Template type 'T' inferred as 'NotAnimal' does not satisfy bound 'Animal'
InvalidTemplateParam@35:4-35:30: Template type 'T' inferred as 'NotAnimal' does not satisfy bound 'Animal'
