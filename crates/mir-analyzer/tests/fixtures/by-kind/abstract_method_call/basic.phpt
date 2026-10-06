===description===
AbstractMethodCall fires when calling an abstract method directly on the class.
===file===
<?php
abstract class Shape {
    abstract public function area(): float;
}

Shape::area();
//<^^^^^^^^^^^^^ AbstractMethodCall: Cannot call abstract method Shape::area()
//<^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Shape::area() cannot be called statically
