===description===
MissingPropertyType fires for promoted constructor parameters that have no type declaration.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
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
