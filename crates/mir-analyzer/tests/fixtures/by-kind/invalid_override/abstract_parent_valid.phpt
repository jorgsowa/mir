===description===
InvalidOverride does NOT fire when #[Override] implements an abstract method declared in an abstract parent class.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
abstract class Base {
    abstract public function render(): void;
}

class Child extends Base {
    #[Override]
    public function render(): void {}
}
