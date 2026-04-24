===file:Shape.php===
<?php
abstract class Shape {
    abstract public function area(): float;
}
===file:Circle.php===
<?php
class Circle extends Shape {
    public function area(): float { return 3.14; }
}
===expect===
