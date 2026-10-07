===description===
Immutability covers instance state only: a static method of an immutable class
may write static properties. An explicitly @pure static method stays flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-immutable */
final class Clock {
    private static ?self $instance = null;
    private static int $created = 0;

    public static function instance(): self {
        return self::$instance ??= new self();
    }

    public static function reset(): int {
        self::$instance = null;
        self::$created++;
        $created = self::$created;
        /** @mir-check $created is int */
        return $created;
    }

    /** @pure */
    public static function replace(self $clock): void {
        self::$instance = $clock;
//      ^^^^^^^^^^^^^^^^^^^^^^^^ ImpureStaticPropertyAssignment: Assigning to static property Clock::$instance in a @pure function
    }
}
