===description===
`self::$prop === []` (and loose `== []`, `static::$prop`) must narrow the
static property to an empty/non-empty collection like the already-correct
instance-property and variable cases — the enum-case/class-const dispatch
guard (any StaticPropertyAccess operand) intercepted this comparison
shape first and silently swallowed it before the array-empty arm ever
ran, in both the strict and loose arms.
===config===
suppress=MissingConstructor,MixedAssignment
===file===
<?php
class Bag {
    /** @var array<int> */
    protected static array $items = [];

    public static function useStrict(): void {
        if (self::$items === []) {
            /** @mir-check self::$items is array{} */
            $_ = 1;
        } else {
            /** @mir-check self::$items is non-empty-array<int> */
            $_ = 1;
        }
    }

    public static function useLoose(): void {
        if (self::$items == []) {
            /** @mir-check self::$items is array{} */
            $_ = 1;
        }
    }
}

class ChildBag extends Bag {
    public static function useViaStatic(): void {
        if (static::$items !== []) {
            /** @mir-check static::$items is non-empty-array<int> */
            $_ = 1;
        }
    }
}
===expect===
