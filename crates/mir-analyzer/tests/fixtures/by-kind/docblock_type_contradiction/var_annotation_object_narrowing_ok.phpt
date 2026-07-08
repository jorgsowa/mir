===description===
Sanity check: `@var` narrowing an object to a subclass (Animal -> Dog) is a
legitimate assertion and must stay silent — both sides map to the same
coarse OBJECT type family, so this is never flagged as a contradiction.
===config===
suppress=UnusedVariable,PossiblyInvalidMethodCall
===file===
<?php
class Animal {}
class Dog extends Animal {
    public function bark(): void {}
}
function f(Animal $animal): void {
    /** @var Dog $animal */
    $animal->bark();
}
===expect===
