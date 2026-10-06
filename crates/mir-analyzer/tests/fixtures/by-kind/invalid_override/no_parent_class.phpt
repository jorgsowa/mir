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
//  ^^^^^^^^^^^ InvalidOverride: Method Orphan::render() has #[Override] but no parent method exists to override
    public function render(): void {}
}
