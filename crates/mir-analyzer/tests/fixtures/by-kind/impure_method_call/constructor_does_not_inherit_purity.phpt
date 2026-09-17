===description===
A subclass constructor does not inherit an ancestor constructor's @pure contract,
but its own @pure contract is enforced.
===file===
<?php
class Reader {
    public function read(): int {
        return 1;
    }
}

class Base {
    /** @pure */
    public function __construct() {}
}

class Child extends Base {
    public function __construct(Reader $reader) {
        $reader->read();
    }
}

class ExplicitPureChild extends Base {
    /** @pure */
    public function __construct(Reader $reader) {
        $reader->read();
    }
}
===expect===
ImpureMethodCall@22:8-22:23: Calling impure method read() in a pure or immutable context
