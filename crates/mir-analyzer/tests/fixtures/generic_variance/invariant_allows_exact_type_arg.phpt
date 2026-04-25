===file===
<?php
/** @template T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Cat {}
/** @param Box<Cat> $b */
function f(Box $b): void { var_dump($b->get()); }
function test(): void {
    /** @var Box<Cat> $c */
    $c = new Box();
    f($c);
}
===expect===
