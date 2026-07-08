===description===
Sanity check: excluding the LOSING trait's copy of a method from override
checking must not disable checking against the WINNING trait's copy.
===file===
<?php
trait T1 { public function f(): int { return 1; } }
trait T2 { public function f(): string { return "x"; } }
class Base {
    use T1, T2 { T2::f insteadof T1; }
}
class Child extends Base {
    public function f(): int { return 1; }
}
===expect===
MethodSignatureMismatch@8:4-8:42: Method Child::f() signature mismatch: return type 'int' is not a subtype of parent 'string'
