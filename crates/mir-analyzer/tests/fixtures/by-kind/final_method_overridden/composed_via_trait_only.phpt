===description===
FN: a method composed purely via `use Trait;` (never redeclared in the class
body) was invisible to every override check — only literally-declared
own_methods() were checked against the parent.
===file===
<?php
class Base {
    final public function foo(): void {}
}
trait T {
    public function foo(): void {}
}
class Child extends Base {
    use T;
}
===expect===
FinalMethodOverridden@6:4-6:34: Method Child::foo() cannot override final method from Base
