===description===
MissingPropertyType fires for promoted constructor parameters that have no type declaration.
===config===
php_version=8.0
===file===
<?php
class Point {
    public function __construct(
        public $x,
//      ^^^^^^^^^ MissingPropertyType: Property Point::$x has no type annotation
        public $y,
//      ^^^^^^^^^ MissingPropertyType: Property Point::$y has no type annotation
    ) {}
}
===expect===
