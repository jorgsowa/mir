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
}
===expect===
OverriddenMethodAccess@9:4-9:37: Method Child::foo() overrides with less visibility
