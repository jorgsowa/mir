===description===
InvalidOverride does NOT fire when #[Override] refers to an existing parent method.
===config===
php_version=8.3
===file===
<?php
class Base {
    public function render(): void {}
}

class Widget extends Base {
    #[\Override]
    public function render(): void {}
}
===expect===
