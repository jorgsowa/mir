===description===
`self::$prop > N` (and `static::$prop`) must narrow the static property's
int range, and a closed-precise range (`int<1,5>`) fully excluded by the
comparison must be recognized as unreachable — the already-correct
instance-property and variable behavior, never wired for a static
property.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Box {
    /** @var int<1,5> */
    protected static int $n = 1;

    public static function useIt(): void {
        if (self::$n > 5) {
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
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}
===expect===
RedundantCondition@7:12-7:24: Condition is always true/false for type 'bool'
RedundantCondition@23:12-23:26: Condition is always true/false for type 'bool'
