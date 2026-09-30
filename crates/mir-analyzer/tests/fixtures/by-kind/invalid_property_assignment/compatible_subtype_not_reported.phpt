===description===
compatible subtype not reported
===file===
<?php
class Animal {}
class Dog extends Animal {}

class Cage {
//<^^^^^^^^^^^^ MissingConstructor: Class Cage has uninitialized properties but no constructor
    public Animal $occupant;
}

$c = new Cage();
$c->occupant = new Dog();
===expect===
