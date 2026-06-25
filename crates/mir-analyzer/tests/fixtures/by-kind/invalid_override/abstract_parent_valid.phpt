===description===
InvalidOverride does NOT fire when #[Override] implements an abstract method declared in an abstract parent class.
===config===
php_version=8.3
===file===
<?php
abstract class Base {
    abstract public function render(): void;
}

class Child extends Base {
    #[Override]
    public function render(): void {}
}
===expect===
