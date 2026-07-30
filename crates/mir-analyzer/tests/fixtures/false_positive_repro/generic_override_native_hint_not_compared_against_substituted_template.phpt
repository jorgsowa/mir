===description===
A class-level `@template T` bound concretely via `@extends Container<Concrete>`
must not force an override that doesn't restate the generic refinement to be
compared against the substituted (narrower) type. The abstract ancestor's own
return type is a bare `@return T`; the override just repeats the ancestor's
plain native hint (`Base`) rather than restating `Concrete` — a standard,
tool-accepted idiom (Psalm/PHPStan don't enforce templates at the native-hint
override-covariance level either). Comparing the child's undecorated native
hint against a substitution it never opted into produced a false
MethodSignatureMismatch ('Base' is not a subtype of 'Concrete').
===config===
suppress=UnusedParam
===file===
<?php
class Base {}
class Concrete extends Base {}

/** @template T of Base */
abstract class Container {
    /** @return T */
    abstract public function get(): Base;
}

/** @extends Container<Concrete> */
final class ConcreteContainer extends Container {
    public function get(): Base {
        return new Concrete();
    }
}
===expect===
