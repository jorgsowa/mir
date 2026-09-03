===description===
Static properties (`self::$prop`/`static::$prop`) got zero narrowing for
bare truthy (`if (self::$prop)`), loose bool (`== true`/`== false`),
`isset()`, and `empty()` — every one of these dispatch arms checked
`extract_var_name`/`extract_any_prop_access` but never
`extract_static_prop_access`, unlike the full strict-equality/
instanceof/is_a/array_key_exists static-property recipe set already
present in this file.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Holder {
    protected static ?string $name = null;

    public static function bareTruthy(): void {
        if (self::$name) {
            /** @mir-check self::$name is non-empty-string */
            $_ = 1;
        }
    }

    public static function looseBoolTrue(): void {
        if (self::$name == true) {
            /** @mir-check self::$name is non-empty-string */
            $_ = 1;
        }
    }

    public static function looseBoolFalse(): void {
        if (self::$name == false) {
            /** @mir-check self::$name is ''|'0'|null */
            $_ = 1;
        }
    }

    public static function issetNarrows(): void {
        if (isset(self::$name)) {
            /** @mir-check self::$name is string */
            $_ = 1;
        }
    }

    public static function emptyNarrows(): void {
        if (!empty(self::$name)) {
            /** @mir-check self::$name is non-empty-string */
            $_ = 1;
        }
    }
}
===expect===
