===description===
An `array<int, T>` return override checks class-hierarchy compatibility of
the array's value type, not just a structural fallback that never
recognizes a subclass as compatible with its declared ancestor.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Unrelated {}

/** @template T */
abstract class Base {
    /** @return array<int, T> */
    abstract public function make(): array;
}

/** @extends Base<Animal> */
class ValidImpl extends Base {
    /** @return array<int, Cat> */
    public function make(): array { return []; }
}

/** @extends Base<Animal> */
class InvalidImpl extends Base {
    /** @return array<int, Unrelated> */
    public function make(): array { return []; }
}
===expect===
MethodSignatureMismatch@21:4-21:48: Method InvalidImpl::make() signature mismatch: return type 'array<int, Unrelated>' is not a subtype of parent 'array<int, Animal>'
