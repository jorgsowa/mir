===description===
Subclass satisfies template bound of parent class
===file===
<?php
class Animal {}
class Dog extends Animal {}

/**
 * @template T of Animal
 * @param T $animal
 */
function adopt($animal): void {
    echo get_class($animal);
}

$dog = new Dog();
adopt($dog);
===expect===
