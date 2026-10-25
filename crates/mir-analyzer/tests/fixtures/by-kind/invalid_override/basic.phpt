===description===
InvalidOverride fires when #[Override] is used but no parent method exists.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
class Base {}

class Child extends Base {
    #[Override]
    public function render(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::render() has #[\Override] attribute, but no matching parent method exists
}
