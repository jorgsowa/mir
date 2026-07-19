===description===
FP: a bare subclass that inherits a `@template-covariant T` without
redeclaring its own `@template` (`class IntBox extends Box {}`) had its
variance silently degraded to invariant, because named_object_subtype looked
up the class's own-declared (empty) template params instead of the effective
(inherited) ones — rejecting a valid covariant argument.
===config===
suppress=ForbiddenCode
===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class BoxChild extends Box {}
class Animal {}
class Cat extends Animal {}
/** @param BoxChild<Animal> $b */
function f(BoxChild $b): void { var_dump($b->get()); }
function test(): void {
    /** @var BoxChild<Cat> $c */
    $c = new BoxChild();
    f($c);
}
===expect===
