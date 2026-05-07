===description===
cross file missing method
===file:Shape.php===
<?php
abstract class Shape {
    abstract public function area(): float;
    abstract public function perimeter(): float;
}
===file:Circle.php===
<?php
class Circle extends Shape {
    public function area(): float { return 3.14; }
}
===expect===
Circle.php: UnimplementedAbstractMethod@2:0: Class Circle must implement abstract method perimeter()
