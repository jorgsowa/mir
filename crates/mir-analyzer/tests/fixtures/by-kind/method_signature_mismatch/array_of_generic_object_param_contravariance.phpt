===description===
An array<int, T>-shaped method-override PARAM checks class-hierarchy
compatibility of the array's value type — is_subtype had no (TArray,TArray)/
(TList,TList) arms at all, so this fell to a pure structural check that
always rejected the pair, even a legal contravariant widening.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Kitten extends Cat {}

/** @template T */
abstract class Base {
    /** @param array<int, T> $items */
    abstract public function process(array $items): void;
}

/** @extends Base<Cat> */
class ValidWideningImpl extends Base {
    /** @param array<int, Animal> $items */
    public function process(array $items): void {}
}

/** @extends Base<Cat> */
class InvalidNarrowingImpl extends Base {
    /** @param array<int, Kitten> $items */
    public function process(array $items): void {}
}
===expect===
MethodSignatureMismatch@21:4-21:50: Method InvalidNarrowingImpl::process() signature mismatch: parameter $items type 'array<int, Kitten>' is narrower than parent type 'array<int, Cat>'
