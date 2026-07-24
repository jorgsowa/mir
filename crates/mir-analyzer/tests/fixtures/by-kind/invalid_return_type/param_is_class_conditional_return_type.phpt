===description===
`@return ($x is X ? A : B)` with a CLASS-typed (not scalar) discriminant
subject — the purely-structural resolver's predicate set only covers
null/true/false/string/list/array/int/float/bool, with no arm for an
object atom at all, so any class-typed subject silently widened to the
union of both branches regardless of the argument's real type.
===config===
suppress=UnusedVariable
===file===
<?php
class Animal {}
class Dog extends Animal {}

/**
 * @param Animal $a
 * @return ($a is Dog ? true : false)
 */
function isDog($a) {
    return $a instanceof Dog;
}

function checkDog(Dog $d): void {
    $x = isDog($d);
    /** @mir-check $x is true */
    $_ = 1;
}

function checkAnimal(Animal $a): void {
    $y = isDog($a);
    /** @mir-check $y is false */
    $_ = 1;
}
===expect===
