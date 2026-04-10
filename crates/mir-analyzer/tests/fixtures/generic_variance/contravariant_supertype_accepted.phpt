===source===
<?php
/**
 * @template-contravariant T
 */
interface Sink {
    /** @param T $item */
    public function accept(mixed $item): void;
}
class Animal {}
class Cat extends Animal {}
/** @param Sink<Cat> $sink */
function takeCatSink(Sink $sink): void {
    $sink->accept(new Cat());
}
function test(): void {
    /** @var Sink<Animal> $animalSink */
    $animalSink = new class implements Sink {
        public function accept(mixed $item): void {}
    };
    takeCatSink($animalSink);
}
===expect===
