===description===
FP: @if-this-is must accept covariant widening for a @template-covariant receiver
===file===
<?php
class Animal {}
class Dog extends Animal {}

/** @template-covariant T */
class Box {
    /** @param T $item */
    public function __construct(private $item) {}

    /** @if-this-is Box<Animal> */
    public function onlyForAnimalBox(): void {}
}

$b = new Box(new Dog());
$b->onlyForAnimalBox();
===expect===
