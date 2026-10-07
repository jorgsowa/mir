===description===
Methods of a @psalm-immutable trait are checked as mutation-free, so calling
them on `$this` or `self::` from an immutable method is allowed and a write to
`$this` inside them is reported where it happens. Plain trait methods stay
flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-immutable */
trait HasCode {
    private int $code = 0;
    private static int $seen = 0;

    public function code(): int {
        return $this->code;
    }

    public function forceCode(int $code): void {
        $this->code = $code;
//      ^^^^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property code of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }

    public static function track(): void {
        self::$seen++;
    }
}

trait PlainCounter {
    private int $hits = 0;

    public function hit(): int {
        return ++$this->hits;
    }
}

/** @psalm-immutable */
final class Status {
    use HasCode;
    use PlainCounter;

    public function label(): string {
        $code = $this->code();
        /** @mir-check $code is int */
        $again = self::code();
        /** @mir-check $again is int */
        return (string) ($code + $again);
    }

    public function counted(): int {
        return $this->hit();
//             ^^^^^^^^^^^^ ImpureMethodCall: Calling impure method hit() in a pure or immutable context
    }
}

final class MutableStatus {
    use HasCode;

    public function bump(): void {
        $this->forceCode($this->code() + 1);
    }
}
