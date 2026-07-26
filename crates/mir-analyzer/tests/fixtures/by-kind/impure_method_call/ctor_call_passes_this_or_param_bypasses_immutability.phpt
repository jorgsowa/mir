===description===
`new X($this)` / `new X($param)` inside an immutable/external-mutation-
free context was never checked against the constructor's own purity —
only a plain @pure function's `new` calls were checked at all. Passing
`$this`/a parameter into a constructor that isn't proven pure/mutation-
free lets it store and later mutate that object, the same risk an
impure METHOD call on it already catches. Contrast with passing a plain
VALUE read off `$this` (not an object), which must stay unflagged — the
standard immutable "wither" idiom.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Wrapper {
    public function __construct(public Config $cfg) {}
}

/** @psalm-immutable */
class Config {
    public int $timeout = 30;

    public function wrapSelf(): Wrapper {
        return new Wrapper($this);
    }

    public function withTimeout(int $timeout): self {
        return new self();
    }
}

class Service {
    /** @psalm-external-mutation-free */
    public function wrapParam(Config $cfg): Wrapper {
        return new Wrapper($cfg);
    }
}
===expect===
ImpureFunctionCall@11:15-11:33: Calling impure function Wrapper::__construct() in a @pure function
ImpureFunctionCall@22:15-22:32: Calling impure function Wrapper::__construct() in a @pure function
