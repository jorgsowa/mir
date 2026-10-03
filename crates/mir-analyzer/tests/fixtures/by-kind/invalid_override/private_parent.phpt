===description===
InvalidOverride fires when #[Override] targets a private parent method (private methods cannot be overridden).
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
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
