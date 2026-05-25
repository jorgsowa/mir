===description===
Multi-level inheritance: grandchild satisfies template bound of ancestor
===file===
<?php
class Animal {}
class Dog extends Animal {}
class Poodle extends Dog {}

/**
 * @template T of Animal
 * @param T $animal
 */
function adopt($animal): void {
    echo get_class($animal);
}

$poodle = new Poodle();
adopt($poodle);
===expect===
