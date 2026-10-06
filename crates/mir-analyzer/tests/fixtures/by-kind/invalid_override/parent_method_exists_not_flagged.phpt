===description===
InvalidOverride does NOT fire when #[Override] refers to an existing parent method.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
class Base {
    public function render(): void {}
}

class Widget extends Base {
    #[\Override]
    public function render(): void {}
}
