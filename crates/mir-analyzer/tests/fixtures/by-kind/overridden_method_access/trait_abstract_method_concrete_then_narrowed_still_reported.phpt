===description===
Negative control for the K4 fix: the carve-out applies only to the trait's
own abstract declaration. Once a class provides the CONCRETE implementation
(satisfying the trait's contract), a further subclass narrowing that
concrete method is a normal override and PHP fatal-errors on it (verified
live) — mir must keep flagging it.
===config===
suppress=MissingConstructor
===file===
<?php
trait PublicRequirement {
    abstract public function foo(): void;
}
class Base {
    use PublicRequirement;
    public function foo(): void {}
}
class Child extends Base {
    protected function foo(): void {}
}
===expect===
OverriddenMethodAccess@10:4-10:37: Method Child::foo() overrides with less visibility
