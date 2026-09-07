===description===
G4: a child may widen an object param (Cat → Animal) — contravariance-legal, no error.
Same-type and widening overrides must not be flagged as narrowing.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Base {
    public function feed(Cat $a): void {}
    public function pet(Animal $a): void {}
}
class Shelter extends Base {
    // widening Cat -> Animal is allowed
    public function feed(Animal $a): void {}
    // same type is allowed
    public function pet(Animal $a): void {}
}
===expect===
