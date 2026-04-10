===source===
<?php
/**
 * @template-covariant T
 */
interface Collection {
    /** @return T */
    public function get(): mixed;
}
class Animal {}
class Cat extends Animal {}
/** @param Collection<Animal> $animals */
function takeAnimals(Collection $animals): void {
    $animals->get();
}
function test(): void {
    /** @var Collection<Cat> $cats */
    $cats = new class implements Collection {
        public function get(): mixed { return new Cat(); }
    };
    takeAnimals($cats);
}
===expect===
