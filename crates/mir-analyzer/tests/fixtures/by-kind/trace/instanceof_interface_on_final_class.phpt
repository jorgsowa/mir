===description===
A final class (or enum) that doesn't implement an interface can't be narrowed to one by `instanceof`; a non-final class, or another interface, intersects with it.
===file===
<?php
interface Shape {}
interface Named {}
final class Circle {}
final class Square implements Shape {}
class Box {}
enum Suit { case Hearts; }

function classes(Circle|Square|Box|Suit $v): void {
    if ($v instanceof Shape) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is Square|Box&Shape
    }
}

function interfaces(Named $v): void {
    if ($v instanceof Shape) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is Named&Shape
    }
}

function finalClassOnInterface(Shape|Box $v): void {
    if ($v instanceof Circle) {
//      ^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        /** @trace $v */
        echo 1;
    }
}
