===description===
FN: a `@param Collection<T> $c` matched against a concrete argument of a
DIFFERENT, more specific class (`TypedList<Dog>` where `TypedList implements
Collection<T>`) never bound `T` at all — `infer_from_pair`'s `TNamedObject`
arm only matched when the arg's class name was EXACTLY the param's declared
class, so `T` fell back to `mixed` instead of the argument's actual type
param, even though the class hierarchy is already walked for subtype checks
elsewhere. `@mir-check` pins the return type at the call site.
===config===
suppress=MissingPropertyType,UnusedParam,MissingThrowsDocblock,UnusedVariable
===file===
<?php
/** @template T */
interface Collection {}

/**
 * @template T
 * @implements Collection<T>
 */
class TypedList implements Collection {
    /** @param T $item */
    public function __construct(private $item) {}
}

class Animal {}
class Dog extends Animal {}

/**
 * @template T
 * @param Collection<T> $c
 * @return T
 */
function firstOf(Collection $c) {
    throw new \Exception();
}

$x = firstOf(new TypedList(new Dog()));
/** @mir-check $x is Dog */
echo "ok";
===expect===
