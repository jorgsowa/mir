===description===
`(self::$prop ?? FALLBACK) !== FALLBACK` (and the loose `!=` sibling)
narrows the static property to non-null, mirroring the existing
plain-variable and instance-property counterparts.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Foo {
    private static ?string $cache = null;

    public static function strictNotDefault(): void {
        if ((self::$cache ?? 'default') !== 'default') {
            /** @mir-check self::$cache is string */
            $_ = self::$cache;
        }
    }

    public static function strictIsDefault(): void {
        if ((self::$cache ?? 'default') === 'default') {
            /** @mir-check self::$cache is string|null */
            $_ = self::$cache;
        }
    }

    public static function looseNotDefault(): void {
        if ((self::$cache ?? 'default') != 'default') {
            /** @mir-check self::$cache is string */
            $_ = self::$cache;
        }
    }
}
===expect===
