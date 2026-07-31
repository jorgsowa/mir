===description===
Negative control for the L11 fix: without a colliding `@psalm-type Parent`
alias in scope, `@return parent` must keep resolving to the real parent
class — the alias lookup must not change genuine keyword behavior.
===config===
suppress=MissingConstructor
===file===
<?php
class Base {}
class Unrelated {}

class Child extends Base {
    /** @return parent */
    public function make(): Base {
        return new Unrelated();
    }
}
===expect===
InvalidReturnType@8:8-8:31: Return type 'Unrelated' is not compatible with declared 'parent(Child)'
