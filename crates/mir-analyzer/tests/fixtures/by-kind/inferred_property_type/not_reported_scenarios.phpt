===description===
Guards for inferred property types: an existing `@var` docblock (or native
type) must never be overridden by inference; a property never assigned
anything stays `mixed`; a property assigned only on ONE branch of an if/else
has no inferred type either — `prop_refined` drops a refinement not present
on every path at the merge point, so there's no signal left to record (a
known, accepted precision gap, not a false positive).
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
class A {}
class B {}

class HasDocblock {
    /** @var A */
    public $thing;

    public function __construct() {
        $this->thing = new B();
    }

    public function read(): void {
        /** @mir-check $this->thing is A */
        $_ = 1;
    }
}

class NeverAssigned {
    public $thing;

    public function read(): void {
        /** @mir-check $this->thing is mixed */
        $_ = 1;
    }
}

class OnlyOneBranchAssigns {
    public $thing;

    public function __construct(bool $cond) {
        if ($cond) {
            $this->thing = new A();
        }
    }

    public function read(): void {
        /** @mir-check $this->thing is mixed */
        $_ = 1;
    }
}
===expect===
InvalidPropertyAssignment@10:8-10:30: Property $thing expects 'A', cannot assign 'B'
