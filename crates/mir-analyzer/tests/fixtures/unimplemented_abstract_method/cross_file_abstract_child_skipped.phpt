===file:Shape.php===
<?php
abstract class Shape {
    abstract public function area(): float;
}
===file:Polygon.php===
<?php
abstract class Polygon extends Shape {
    # area() still abstract — abstract child is fine
}
===file:Triangle.php===
<?php
class Triangle extends Polygon {
    public function area(): float { return 0.5; }
}
===expect===
