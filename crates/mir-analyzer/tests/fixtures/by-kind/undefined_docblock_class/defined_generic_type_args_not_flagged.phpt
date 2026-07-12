===description===
UndefinedDocblockClass does NOT fire for `@extends`/`@implements` generic
type arguments, or a `@template ... of Bound`, that name real classes —
and a class forwarding its own template param positionally to its parent
(`class TypedList<T> implements Collection<T>`) is not mistaken for a
reference to an undefined class named "T".
===config===
suppress=UnusedParam,MissingReturnType,MissingParamType
===file===
<?php
class Animal {}

/** @template T of Animal */
class Box {}

/** @extends Box<Animal> */
class AnimalBox extends Box {}

/**
 * @template TKey
 * @template TValue
 */
interface Collection {
    public function get($key);
}

/**
 * @template T
 * @implements Collection<int, T>
 */
class TypedList implements Collection {
    public function get($key) {
        return null;
    }
}
===expect===
