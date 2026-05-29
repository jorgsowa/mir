===description===
covariant rejects supertype arg
===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Animal {}
class Cat extends Animal {}
/** @param Box<Cat> $b */
function f(Box $b): void { var_dump($b->get()); }
function test(): void {
    /** @var Box<Animal> $a */
    $a = new Box();
    f($a);
}
===expect===
InvalidArgument@14:7-14:9: Argument $b of f() expects 'Box<Cat>', got 'Box<Animal>'
