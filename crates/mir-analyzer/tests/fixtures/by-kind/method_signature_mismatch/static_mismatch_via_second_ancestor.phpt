===description===
FN: static/non-static mismatch only checked `all_parent_methods.first()`
(the parent class) — a conflicting interface further down the ancestor
chain was silently skipped.
===file===
<?php
class Base {
    public static function foo(): void {}
}
interface Iface {
    public function foo(): void;
}
class Child extends Base implements Iface {
    public static function foo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::foo() signature mismatch: cannot override non-static method Iface::foo() with a static method
}
