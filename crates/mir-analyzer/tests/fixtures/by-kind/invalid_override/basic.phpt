===description===
InvalidOverride fires when #[Override] is used but no parent method exists.
===config===
php_version=8.3
===file===
<?php
class Base {}

class Child extends Base {
    #[Override]
//  ^^^^^^^^^^^ InvalidOverride: Method Child::render() has #[Override] but no parent method exists to override
    public function render(): void {}
}
===expect===
