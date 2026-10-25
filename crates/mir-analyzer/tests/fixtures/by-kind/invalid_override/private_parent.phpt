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
    public function render(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::render() has #[\Override] attribute, but no matching parent method exists
}
