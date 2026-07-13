===description===
A trait-composed method returning `static` must not be falsely flagged
against a parent also returning `static` — both resolve to the same class
at the actual use site, so `self`/`static` in the trait's own signature must
be rebound to the composing class before comparison.
===file===
<?php
class Base {
    public static function make(): static {
        return new static();
    }
}
trait T {
    public static function make(): static {
        return new static();
    }
}
class Child extends Base {
    use T;
}
===expect===
