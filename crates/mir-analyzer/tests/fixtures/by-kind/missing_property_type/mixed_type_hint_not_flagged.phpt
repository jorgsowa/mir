===description===
MissingPropertyType does NOT fire for properties declared with the 'mixed' type — explicit mixed is a valid native type in PHP 8.0+.
===config===
php_version=8.0
===file===
<?php
class Container {
    public function __construct(
        public mixed $data = null,
        private mixed $cache = null,
    ) {}
}
===expect===
