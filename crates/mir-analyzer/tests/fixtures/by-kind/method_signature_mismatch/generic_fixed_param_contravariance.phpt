===description===
Parameter contravariance for an object-typed generic param concretely
fixed via @extends is now enforced, mirroring the existing return-type
covariance carve-out — the parent's own docblock `@param T` type isn't
exempt just because it's docblock-only, once this class's `@extends`
concretely binds T. A plain, non-generic docblock narrowing (native
hint unchanged) stays exempt, same as before.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Animal {}
class Cat extends Animal {}

/** @template T */
class Box {
    /** @param T $x */
    public function set($x): void {}
}

/** @extends Box<Animal> */
class AnimalBox extends Box {
    /** @param Cat $x */
    public function set($x): void {}
}

interface Renderer {
    public function draw(Animal $shape): void;
}

class AnimalRenderer implements Renderer {
    /** @param Cat $shape */
    public function draw(Animal $shape): void {}
}
===expect===
MethodSignatureMismatch@14:4-14:36: Method AnimalBox::set() signature mismatch: parameter $x type 'Cat' is narrower than parent type 'Animal'
