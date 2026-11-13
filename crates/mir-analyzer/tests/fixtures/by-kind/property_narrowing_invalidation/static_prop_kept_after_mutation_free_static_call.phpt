===description===
A static property's narrowing survives a call to a same-class static method that writes
no property, and is dropped by one that does.
===file===
<?php
final class C {
    private static ?array $cache = null;

    private static function other(): ?int { return null; }
    private static function reset(): void { self::$cache = null; }

    public static function get(): array {
        self::$cache = ['a' => 1];
        self::other();
        return self::$cache;
    }

    public static function getAfterReset(): array {
        self::$cache = ['a' => 1];
        self::reset();
        return self::$cache;
//      ^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'array|null' is not compatible with declared 'array'
    }
}
