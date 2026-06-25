===description===
InvalidOverride fires when #[Override] is used in a class with no parent class at all.
===config===
php_version=8.3
===file===
<?php
class Orphan {
    #[Override]
    public function render(): void {}
}
===expect===
InvalidOverride@3:4-3:15: Method Orphan::render() has #[Override] but no parent method exists to override
