===description===
Wrong case method name on an inherited method is reported.
===file===
<?php
class Animal {
    public function makeSound(): void {}
}
class Dog extends Animal {}
$d = new Dog();
$d->MAKESOUND();
//  ^^^^^^^^^ WrongCaseMethod: Method name 'Dog::MAKESOUND' has incorrect casing; use 'makeSound'
