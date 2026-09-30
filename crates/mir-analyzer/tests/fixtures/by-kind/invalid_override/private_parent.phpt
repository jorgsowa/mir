===description===
InvalidOverride fires when #[Override] targets a private parent method (private methods cannot be overridden).
===config===
php_version=8.3
===file===
<?php
class Base {
    private function render(): void {}
}

class Child extends Base {
    #[Override]
//  ^^^^^^^^^^^ InvalidOverride: Method Child::render() has #[Override] but parent method Base::render() is private
    public function render(): void {}
}
===expect===
