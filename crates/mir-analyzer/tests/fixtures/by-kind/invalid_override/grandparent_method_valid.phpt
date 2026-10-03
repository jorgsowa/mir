===description===
InvalidOverride does NOT fire when #[Override] refers to a method on a grandparent class (ancestor chain is searched transitively).
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
class GrandParent {
    public function render(): void {}
}

class Middle extends GrandParent {}

class Child extends Middle {
    #[\Override]
    public function render(): void {}
}
===expect===
