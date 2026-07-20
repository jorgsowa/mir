===description===
`if (self::$prop = expr)` (assignment-as-condition) narrows the static
property to truthy/falsy, mirroring the existing plain-variable and
instance-property counterparts.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Foo {
    private static ?string $cache = null;

    public static function compute(): string {
        if (self::$cache = self::fetch()) {
            /** @mir-check self::$cache is non-empty-string */
            return self::$cache;
        } else {
            /** @mir-check self::$cache is ''|'0'|null */
            return 'default';
        }
    }

    private static function fetch(): ?string {
        return null;
    }
}
===expect===
