===description===
`is_string(self::$prop)` / `is_numeric(static::$prop)` (and
`method_exists` on a static-prop receiver) must narrow the static
property like the already-correct instance-property and variable cases —
narrow_prop_from_type_fn's dispatch never checked
extract_static_prop_access.
===config===
suppress=MissingConstructor,MissingPropertyType,PossiblyNullArgument
===file===
<?php
class Box {
    /** @var int|string */
    protected static $value = 0;

    public static function useIsString(): void {
        if (is_string(self::$value)) {
            /** @mir-check self::$value is string */
            echo strlen(self::$value);
        }
    }

    public static function useIsNumeric(): int|float {
        if (is_numeric(static::$value)) {
            return static::$value + 0;
        }
        return 0;
    }
}

class WithMethod {
    protected static ?WithMethod $other = null;

    public static function useMethodExists(): void {
        if (method_exists(self::$other, 'useMethodExists')) {
            self::$other->useMethodExists();
        }
    }
}
===expect===
