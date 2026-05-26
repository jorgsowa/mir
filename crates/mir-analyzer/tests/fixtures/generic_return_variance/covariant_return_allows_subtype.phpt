===description===
Covariant template param accepts a subtype in a return statement.
Regression: return_type_params_compatible used to do structural-only
subtype checks, which rejected Box<Cat> as Box<Animal> even though
Cat extends Animal and Box's T is covariant.
===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Animal {}
class Cat extends Animal {}

/** @return Box<Animal> */
function make(): Box {
    /** @var Box<Cat> $b */
    $b = new Box();
    return $b;
}
===expect===
