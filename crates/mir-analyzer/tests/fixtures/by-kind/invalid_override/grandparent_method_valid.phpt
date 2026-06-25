===description===
InvalidOverride does NOT fire when #[Override] refers to a method on a grandparent class (ancestor chain is searched transitively).
===config===
php_version=8.3
===file===
<?php
class GrandParent {
    public function render(): void {}
}

class Middle extends GrandParent {}

class Child extends Middle {
    #[\Override]
    public function render(): void {}
}
===expect===
