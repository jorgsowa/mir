===description===
Gap: __toString() in a trait returning int is NOT flagged — trait method bodies have check_returns: false
===file===
<?php
trait ToStringTrait {
    public function __toString(): int {
        return 42;
    }
}
class Foo {
    use ToStringTrait;
}
new Foo();
===expect===
