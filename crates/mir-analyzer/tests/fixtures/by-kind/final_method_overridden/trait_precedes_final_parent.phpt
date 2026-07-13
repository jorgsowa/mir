===description===
FN: only the first ancestor (always a trait, ordered before the real parent)
was checked for final-override — a trait's compatible copy shadowed a real
conflict against the parent class further down the chain.
===file===
<?php
trait T {
    public function foo(): void {}
}
class Base {
    final public function foo(): void {}
}
class Child extends Base {
    use T;
    public function foo(): void {}
}
===expect===
FinalMethodOverridden@10:4-10:34: Method Child::foo() cannot override final method from Base
