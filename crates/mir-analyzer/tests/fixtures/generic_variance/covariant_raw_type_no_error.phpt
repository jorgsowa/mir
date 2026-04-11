===source===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Animal {}
/** @param Box<Animal> $b */
function f(Box $b): void { var_dump($b->get()); }
function test(): void {
    $raw = new Box();
    f($raw);
}
===expect===

