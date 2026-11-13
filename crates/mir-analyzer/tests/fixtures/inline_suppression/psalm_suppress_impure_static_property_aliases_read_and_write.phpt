===description===
Psalm's `ImpureStaticProperty` covers both reading and assigning a static property
in a pure scope, mir's `ImpureStaticPropertyAccess` and `ImpureStaticPropertyAssignment`.
===file===
<?php
final class C {
    private static ?array $cache = null;

    /**
     * @psalm-pure
     * @psalm-suppress ImpureStaticProperty
     */
    public static function get(): array {
        return self::$cache ??= [1];
    }
}
