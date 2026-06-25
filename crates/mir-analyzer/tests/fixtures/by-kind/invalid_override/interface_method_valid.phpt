===description===
InvalidOverride does NOT fire when #[Override] implements a method declared in an interface.
===config===
php_version=8.3
===file===
<?php
interface Renderable {
    public function render(): void;
}

class Widget implements Renderable {
    #[Override]
    public function render(): void {}
}
===expect===
