===description===
InvalidOverride fires when #[Override] is used in a class with no parent class at all.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
class Orphan {
    #[Override]
    public function render(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Orphan::render() has #[\Override] attribute, but no matching parent method exists
}
