===description===
compatible subtype not reported
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
MissingConstructor@5:0-5:12: Class Cage has uninitialized properties but no constructor
