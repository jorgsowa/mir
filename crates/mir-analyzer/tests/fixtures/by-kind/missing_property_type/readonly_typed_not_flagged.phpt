===description===
MissingPropertyType does NOT fire for PHP 8.1 readonly promoted constructor parameters that have a type declaration.
===config===
php_version=8.1
===file===
<?php
class ImmutablePoint {
    public function __construct(
        public readonly float $x,
        public readonly float $y,
        public readonly float $z = 0.0,
    ) {}
}
===expect===
