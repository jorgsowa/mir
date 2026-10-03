===description===
InvalidOverride does NOT fire when #[Override] overrides a protected parent method (only private methods are unoverridable).
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
class Base {
    protected function render(): void {}
}

class Child extends Base {
    #[Override]
    protected function render(): void {}
}
===expect===
