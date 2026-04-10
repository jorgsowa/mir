===source===
<?php
/**
 * @template-covariant T
 */
class ReadOnlyList {
    /** @return T */
    public function first(): mixed { return null; }
}
class Animal {}
class Cat extends Animal {}
function test(): void {
    /** @var ReadOnlyList<Cat> $cats */
    $cats = new ReadOnlyList();
    /** @var ReadOnlyList<Animal> $animals */
    $animals = $cats;
    $animals->first();
}
===expect===
