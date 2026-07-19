===description===
`is_subclass_of($x, 'Animal')` on a union that includes an intersection
member (`(Foo&Dog)|Cat`) must not silently drop that member — before this
fix `narrow_strict_subclass_of` had no `TIntersection` arm at all, so it
fell through the catch-all and vanished from the narrowed union entirely,
same soundness bug the `instanceof` sibling already had fixed.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Foo {}
class Animal {}
class Dog extends Animal {}
class Cat extends Animal {}

/** @param (Foo&Dog)|Cat $x */
function alreadyProvenSubclassIsKept($x): void {
    if (is_subclass_of($x, 'Animal')) {
        /** @mir-check $x is (Foo&Dog)|Cat */
        $_ = 1;
    }
}

interface Bar {}

/** @param (Foo&Bar)|Cat $x */
function unrelatedIntersectionGetsExtended($x): void {
    if (is_subclass_of($x, 'Animal')) {
        /** @mir-check $x is (Foo&Bar&Animal)|Cat */
        $_ = 1;
    }
}
===expect===
