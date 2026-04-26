===file===
<?php
class Animal {}
class Dog extends Animal {}

class Cage {
    public Animal $occupant;
}

$c = new Cage();
$c->occupant = new Dog();
===expect===
