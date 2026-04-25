===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Animal {}
class Cat extends Animal {}
/** @param Box<Animal> $b */
function f(Box $b): void { var_dump($b->get()); }
function test(): void {
    /** @var Box<Cat> $c */
    $c = new Box();
    f($c);
}
===expect===
