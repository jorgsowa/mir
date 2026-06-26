===description===
MissingPropertyType fires for promoted constructor parameters that have no type declaration.
===config===
php_version=8.0
===file===
<?php
class Point {
    public function __construct(
        public $x,
        public $y,
    ) {}
}
===expect===
MissingPropertyType@4:8-4:17: Property Point::$x has no type annotation
MissingPropertyType@5:8-5:17: Property Point::$y has no type annotation
