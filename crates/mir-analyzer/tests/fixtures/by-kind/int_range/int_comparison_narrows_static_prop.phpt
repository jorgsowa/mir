===description===
`self::$prop > N` (and `static::$prop`) must narrow the static property's
int range, and a closed-precise range (`int<1,5>`) fully excluded by the
comparison must be recognized as unreachable — the already-correct
instance-property and variable behavior, never wired for a static
property.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    /** @var int<1,5> */
    protected static int $n = 1;

    public static function useIt(): void {
        if (self::$n > 5) {
//          ^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }

    public static function useItReachable(): void {
        if (self::$n > 3) {
            /** @mir-check self::$n is int<4, 5> */
            $_ = 1;
        }
    }
}

class ChildBox extends Box {
    public static function useItViaStatic(): void {
        if (static::$n > 5) {
//          ^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}
