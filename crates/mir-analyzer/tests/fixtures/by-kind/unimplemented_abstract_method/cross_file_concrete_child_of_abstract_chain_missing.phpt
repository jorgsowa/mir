===description===
cross file concrete child of abstract chain missing
===file:Shape.php===
<?php
abstract class Shape {
    abstract public function area(): float;
}
===file:Polygon.php===
<?php
abstract class Polygon extends Shape {}
===file:Triangle.php===
<?php
class Triangle extends Polygon {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedAbstractMethod: Class Triangle must implement abstract method area()
    # area() NOT implemented despite being required
}
===expect===
