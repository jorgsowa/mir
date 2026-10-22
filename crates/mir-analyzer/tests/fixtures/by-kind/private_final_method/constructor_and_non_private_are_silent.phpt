===description===
PHP exempts the constructor, and `final` matters on protected and public methods.
===file===
<?php
class A {
    final private function __construct() {}
    final protected function f(): void {}
    final public static function make(): self { return new self(); }
}
