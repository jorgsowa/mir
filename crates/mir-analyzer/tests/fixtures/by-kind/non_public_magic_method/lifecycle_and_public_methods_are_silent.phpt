===description===
The constructor, destructor and `__clone` may be non-public, and public or implicitly public magic methods are fine.
===file===
<?php
final class Singleton {
    private function __construct() {}
    private function __clone() {}
    protected function __destruct() {}

    function __get(string $name): mixed { return $name; }
    public function __set(string $name, mixed $value): void {}
}
