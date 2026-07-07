===description===
FN: contravariance/covariance checks were skipped outright whenever the
parent's param/return type mentioned a template, even when this class's own
`@extends Box<int>` concretely bound that template. Substituting the
inherited binding into the ancestor's type before comparing now catches a
real signature narrowing that was previously silent.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @param T $x */
    public function set($x): void {}

    /** @return T */
    public function get() {
        return null;
    }
}

/** @extends Box<int> */
class IntBox extends Box {
    public function set(string $x): void {}
}

/** @extends Box<int> */
class StringBox extends Box {
    public function get(): string {
        return "x";
    }
}
===expect===
MethodSignatureMismatch@17:4-17:43: Method IntBox::set() signature mismatch: parameter $x type 'string' is narrower than parent type 'int'
MethodSignatureMismatch@22:4-22:35: Method StringBox::get() signature mismatch: return type 'string' is not a subtype of parent 'int'
