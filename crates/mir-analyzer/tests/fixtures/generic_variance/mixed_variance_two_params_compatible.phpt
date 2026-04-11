===source===
<?php
/**
 * @template K
 * @template-covariant V
 */
class Pair {}
class Animal {}
class Cat extends Animal {}
/** @param Pair<string, Animal> $p */
function f(Pair $p): void { var_dump($p); }
function test(): void {
    /** @var Pair<string, Cat> $p */
    $p = new Pair();
    f($p);
}
===expect===

