===description===
FALSE POSITIVE reproducer. Valid PHP: `use BaseInit { __construct as __constructBase; }` makes `__constructBase` a real method.
mir 0.42.0 currently emits (the bug): UndefinedMethod@9:8-9:33: Query::__constructBase
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
trait BaseInit {
    public function __construct(int $x) {}
}
class Query {
    use BaseInit { __construct as __constructBase; }
    public function __construct() {
        // FP expected: UndefinedMethod __constructBase (trait aliasing not tracked)
        $this->__constructBase(1);
    }
}
===expect===
