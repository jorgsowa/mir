===description===
Non static interface method
===file===
<?php
interface I {
    public static function m(): void;
}
class C implements I {
    public function m(): void {}
}
===expect===
MethodSignatureMismatch@6:4-6:32: Method C::m() signature mismatch: cannot override static method I::m() with a non-static method
