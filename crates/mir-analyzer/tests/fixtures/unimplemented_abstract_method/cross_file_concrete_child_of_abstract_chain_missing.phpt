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
    # area() NOT implemented despite being required
}
===expect===
Triangle.php: UnimplementedAbstractMethod: Class Triangle must implement abstract method area()
