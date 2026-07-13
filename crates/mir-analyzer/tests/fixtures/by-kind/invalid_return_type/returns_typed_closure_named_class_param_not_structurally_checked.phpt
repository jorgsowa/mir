===description===
A named-class param/return in a Closure(T):R signature is skipped by the
purely-structural TClosure<:TClosure check (no db access to resolve real
inheritance) — deliberately silent rather than risking a false positive
on a legitimate subclass/superclass substitution. Scalar mismatches
(returns_typed_closure_with_bad_param_type/_return_type) still fire.
===file===
<?php
class C {}
class C2 extends C {}
class A {}
class A2 extends A {}

/**
 * @return Closure(C2):A
 */
function foo(): Closure {
    return function (C $x): A2 {
        return new A2();
    };
}
===expect===
