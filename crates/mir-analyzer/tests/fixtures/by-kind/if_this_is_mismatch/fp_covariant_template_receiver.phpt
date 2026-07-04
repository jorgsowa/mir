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
MissingPropertyType@8:32-8:45: Property Box::$item has no type annotation
