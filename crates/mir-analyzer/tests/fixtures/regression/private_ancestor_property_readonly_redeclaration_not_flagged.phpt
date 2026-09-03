===description===
Child redeclares a same-named property as non-readonly against a PRIVATE ancestor
property of different readonly-ness — legal PHP, since a private property isn't
inherited and the child's declaration is an unrelated one.
===file===
<?php
class Base {
    public function __construct(private readonly int $x) {
    }
}

class Sub extends Base {
    public function __construct(private int $x) {
    }
}
===expect===
