===description===
FN: visibility-reduction only checked `all_parent_methods.first()` (the
parent class) — reducing below an interface's public contract further down
the ancestor chain was silently skipped.
===file===
<?php
class Base {
    protected function foo(): void {}
}
interface Iface {
    public function foo(): void;
}
class Child extends Base implements Iface {
    protected function foo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Access level to Child::foo() must be public (as in class Iface)
}
