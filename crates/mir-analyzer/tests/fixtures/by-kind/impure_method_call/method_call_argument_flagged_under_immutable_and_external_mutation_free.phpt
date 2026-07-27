===description===
An instance method call's own receiver-based immutable/external-mutation-
free checks only covered the call's OWN object — a plain, by-value
argument reachable from $this/a parameter passed into a not-provably-safe
callee, through a receiver that's itself safe (not $this, not a
parameter), went completely unflagged, unlike new X(...), free-function
calls, and static calls.
===config===
suppress=MissingConstructor,MixedArgument
===file===
<?php
class Box {
    public bool $touched = false;
}

class Logger {
    public function record(Box $o): void {
        $o->touched = true;
    }
}

/** @psalm-immutable */
class Holder {
    public Box $box;

    public function corruptThis(): void {
        $logger = new Logger();
        $logger->record($this->box);
    }
}

class Wrapper {
    /** @psalm-external-mutation-free */
    public function corruptParam(Box $box): void {
        $logger = new Logger();
        $logger->record($box);
    }

    /** @psalm-external-mutation-free */
    public function safeValue(): void {
        $logger = new Logger();
        $logger->record(new Box());
    }
}
===expect===
ImpureMethodCall@18:8-18:35: Calling impure method record() in a pure or immutable context
ImpureMethodCall@26:8-26:29: Calling impure method record() in a pure or immutable context
