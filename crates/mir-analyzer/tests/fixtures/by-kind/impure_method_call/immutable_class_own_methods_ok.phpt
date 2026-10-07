===description===
Methods declared on an immutable class are mutation-free, so calling them on
`$this`, `self::` or `parent::` from an immutable method is allowed. Methods
inherited from a mutable parent or a plain trait, the constructor, and calls on
a mutable dependency stay flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
    <DirectConstructorCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Counter {
    public int $n = 0;

    public function bump(): void {
        $this->n++;
    }
}

class MutableBase {
    public int $m = 0;

    public function touch(): int {
        return ++$this->m;
    }
}

trait PlainHelpers {
    public function one(): int {
        return 1;
    }
}

/** @psalm-immutable */
class Money extends MutableBase {
    use PlainHelpers;

    public function __construct(private int $amount, private Counter $counter) {}

    public function amount(): int {
        return $this->amount;
    }

    public function doubled(): int {
        $a = $this->amount();
        /** @mir-check $a is int */
        $b = self::amount();
        /** @mir-check $b is int */
        return $a + $b;
    }

    public function inherited(): int {
        return $this->touch();
//             ^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method touch() in a pure or immutable context
    }

    public function viaTrait(): int {
        return $this->one();
//             ^^^^^^^^^^^^ ImpureMethodCall: Calling impure method one() in a pure or immutable context
    }

    public function reset(): void {
        $this->__construct(0, $this->counter);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method __construct() in a pure or immutable context
    }

    public function deep(): void {
        $this->counter->bump();
//      ^^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method bump() in a pure or immutable context
    }
}

/** @psalm-immutable */
final class Euro extends Money {
    public function viaParent(): int {
        $p = parent::amount();
        /** @mir-check $p is int */
        return $p + $this->doubled();
    }
}
