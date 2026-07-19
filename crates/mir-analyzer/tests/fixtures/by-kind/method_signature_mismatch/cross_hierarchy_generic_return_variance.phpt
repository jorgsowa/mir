===description===
A method override returning a different-but-related generic class (`SubBox`
extends `Box`) now checks variance on the type params, not just class
inheritance — `named_object_return_compatible`'s inheritance-check branch
previously proved `SubBox<Cat>` compatible with a declared `Box<Animal>`
purely from `SubBox extends Box`, ignoring that `Box`'s invariant `T` makes
`Cat` and `Animal` incompatible.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Animal {}
class Cat extends Animal {}

/** @template T */
class Box {}
class SubBox extends Box {}

/** @template T */
abstract class Base {
    /** @return Box<T> */
    abstract public function make(): Box;
}

/** @extends Base<Animal> */
class MismatchedImpl extends Base {
    /** @return SubBox<Cat> */
    public function make(): SubBox { return new SubBox(); }
}

/** @extends Base<Animal> */
class MatchingImpl extends Base {
    /** @return SubBox<Animal> */
    public function make(): SubBox { return new SubBox(); }
}

/** @template-covariant T */
class CovariantBox {}
class SubCovariantBox extends CovariantBox {}

/** @template T */
abstract class CovariantBase {
    /** @return CovariantBox<T> */
    abstract public function make(): CovariantBox;
}

/** @extends CovariantBase<Animal> */
class CovariantImpl extends CovariantBase {
    /** @return SubCovariantBox<Cat> */
    public function make(): SubCovariantBox { return new SubCovariantBox(); }
}
===expect===
MethodSignatureMismatch@18:4-18:59: Method MismatchedImpl::make() signature mismatch: return type 'SubBox<Cat>' is not a subtype of parent 'Box<Animal>'
