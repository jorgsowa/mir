===description===
MissingPropertyType does NOT fire for promoted constructor parameters that have a type declaration.
===config===
php_version=8.0
===file===
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y,
        protected int $count = 0,
        private ?string $label = null,
    ) {}
}
===expect===
