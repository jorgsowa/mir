===description===
An instance method call's own receiver-based immutable/external-mutation-
free checks only covered the call's OWN object — a plain, by-value
argument reachable from $this/a parameter passed into a not-provably-safe
callee, through a receiver that's itself safe (not $this, not a
parameter), went completely unflagged, unlike new X(...), free-function
calls, and static calls.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method record() in a pure or immutable context
    }
}

class Wrapper {
    /** @psalm-external-mutation-free */
    public function corruptParam(Box $box): void {
        $logger = new Logger();
        $logger->record($box);
//      ^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method record() in a pure or immutable context
    }

    /** @psalm-external-mutation-free */
    public function safeValue(): void {
        $logger = new Logger();
        $logger->record(new Box());
    }
}
===expect===
