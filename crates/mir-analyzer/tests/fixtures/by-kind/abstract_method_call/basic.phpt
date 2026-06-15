===description===
AbstractMethodCall fires when calling an abstract method directly on the class.
===file===
<?php
abstract class Shape {
    abstract public function area(): float;
}

Shape::area();
===expect===
AbstractMethodCall@6:0-6:13: Cannot call abstract method Shape::area()
InvalidStaticInvocation@6:0-6:13: Non-static method Shape::area() cannot be called statically
