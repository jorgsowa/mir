===description===
Non static interface method
===file===
<?php
interface I {
    public static function m(): void;
}
class C implements I {
    public function m(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method C::m() signature mismatch: cannot override static method I::m() with a non-static method
}
