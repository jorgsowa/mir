===description===
Negative control for the K4 fix: the trait-abstract-method carve-out must
not leak to an abstract CLASS's abstract method — PHP fatal-errors on this
(verified live), so mir must keep flagging it.
===file===
<?php
abstract class Base {
    abstract public function foo(): void;
}
class Impl extends Base {
    protected function foo(): void {}
}
===expect===
OverriddenMethodAccess@6:4-6:37: Method Impl::foo() overrides with less visibility
