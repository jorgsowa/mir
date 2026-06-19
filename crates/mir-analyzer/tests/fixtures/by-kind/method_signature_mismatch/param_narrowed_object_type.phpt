===description===
G4: a child illegally narrows an object param from Animal to Cat (contravariance
violation) — must emit MethodSignatureMismatch.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Base {
    public function feed(Animal $a): void {}
}
class Kitten extends Base {
    public function feed(Cat $a): void {}
}
===expect===
MethodSignatureMismatch@8:4-8:41: Method Kitten::feed() signature mismatch: parameter $a type 'Cat' is narrower than parent type 'Animal'
