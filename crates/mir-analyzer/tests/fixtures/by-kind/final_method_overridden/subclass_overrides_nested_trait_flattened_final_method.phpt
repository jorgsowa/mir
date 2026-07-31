===description===
Negative control for the K5 fix: a real subclass overriding a method that
became final only via a transitively-composed (nested) trait must still be
flagged — this is genuine inheritance, unlike the flattening class's own
composition (verified live: PHP fatals here).
===file===
<?php
trait Inner {
    final public function f(): void {}
}
trait Outer {
    use Inner;
}
class C {
    use Outer;
}
class D extends C {
    public function f(): void {}
}
===expect===
FinalMethodOverridden@12:4-12:32: Method D::f() cannot override final method from Inner
