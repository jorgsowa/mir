===description===
Known scope limit, not a false-negative regression: an INHERITED property is
never checked by this pass, even when the child's own constructor neither
calls `parent::__construct()` nor assigns it itself. Initializing an
inherited property is the declaring ancestor's own constructor's contract;
checking it here would require modeling what the parent constructor
guarantees, which this pass does not attempt.
===file===
<?php
class Base {
    public string $inherited;

    public function __construct(string $v) {
        $this->inherited = $v;
    }
}

class Child extends Base {
    public string $own;

    public function __construct(string $v) {
        $this->own = $v;
    }
}
===expect===
