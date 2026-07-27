===description===
A subclass genuinely overriding a method that became final only via the
parent's `use`-composed trait (no own override in the parent) must still be
flagged — this is real inheritance, unlike the parent's own composition
(verified live: PHP fatals here).
===file===
<?php
trait Greeter {
    final protected function greet(): string { return 'hi'; }
}
class A {
    use Greeter;
}
class B extends A {
    protected function greet(): string { return 'bye'; }
}
===expect===
FinalMethodOverridden@9:4-9:56: Method B::greet() cannot override final method from Greeter
