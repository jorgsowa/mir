===description===
FinalMethodOverridden fires when a grandparent's method is final and a grandchild overrides it.
===file===
<?php
class Grandparent {
    final public function locked(): void {}
}
class Middle extends Grandparent {}
class Child extends Middle {
    public function locked(): void {}
}
===expect===
FinalMethodOverridden@7:4-7:37: Method Child::locked() cannot override final method from Grandparent
