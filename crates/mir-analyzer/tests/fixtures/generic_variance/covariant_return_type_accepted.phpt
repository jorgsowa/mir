===source===
<?php
/**
 * @template-covariant T
 */
interface Box {
    /** @return T */
    public function get(): mixed;
}
class Animal {}
class Cat extends Animal {}
/** @return Box<Cat> */
function getCatBox(): Box {
    /** @var Box<Cat> $b */
    $b = new class implements Box {
        public function get(): mixed { return new Cat(); }
    };
    return $b;
}
/** @return Box<Animal> */
function getAnimalBox(): Box {
    return getCatBox();
}
===expect===
