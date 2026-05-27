===description===
TIntersection as both sub and sup — a value declared as A&B satisfies a param expecting A&B
===file===
<?php
namespace App;

interface A {}
interface B {}
class C implements A, B {}

/**
 * @template T of A&B
 * @param T $t
 */
function f($t): void {
    $t;
}

f(new C());
===expect===
