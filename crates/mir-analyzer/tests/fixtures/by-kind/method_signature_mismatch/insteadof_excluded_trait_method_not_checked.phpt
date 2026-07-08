===description===
FP: MethodSignatureMismatch fired against a trait method that `insteadof`
explicitly excluded from the using class's effective method set — the
ancestor walk treated the losing trait as a real "parent" to check against.
===file===
<?php
trait T1 { public function f(): int { return 1; } }
trait T2 { public function f(): string { return "x"; } }
class Base {
    use T1, T2 { T2::f insteadof T1; }
}
class Child extends Base {
    public function f(): string { return "y"; }
}
===expect===
