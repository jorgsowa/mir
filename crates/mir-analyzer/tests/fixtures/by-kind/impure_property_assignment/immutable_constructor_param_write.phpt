===description===
A @psalm-immutable constructor may initialize `$this` but must not write to
its arguments' properties, directly, through a chain, or from a closure. Reading
an argument, passing it on, and writing a local object stay allowed, as does
any write in a mutable class's constructor.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Inner {
    public int $v = 0;
}

final class Counter {
    public int $n = 0;
    /** @var list<int> */
    public array $log = [];
    public Inner $inner;

    public function __construct() {
        $this->inner = new Inner();
    }

    public function get(): int {
        return $this->n;
    }
}

final class Holder {
    public function __construct(public Counter $c) {}
}

/** @psalm-immutable */
final class Snapshot {
    private int $n;
    private Holder $holder;

    public function __construct(Counter $counter, Counter $other) {
        $this->n = $counter->get();
        /** @mir-check $this->n is int */
        $this->holder = new Holder($counter);
        $local = new Counter();
        $local->n = 1;

        $counter->n = 5;
//      ^^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property n of a parameter in a pure or external-mutation-free context
        $counter->inner->v = 1;
//      ^^^^^^^^^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property v of a parameter in a pure or external-mutation-free context
        $counter->log[] = 1;
//      ^^^^^^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property log of a parameter in a pure or external-mutation-free context
        $counter->n++;
//      ^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property n of a parameter in a pure or external-mutation-free context
        $reset = function () use ($other): void {
            $other->n = 0;
//          ^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property n of a parameter in a pure or external-mutation-free context
        };
    }
}

final class Recorder {
    public function __construct(Counter $counter) {
        $counter->n = 5;
    }
}
