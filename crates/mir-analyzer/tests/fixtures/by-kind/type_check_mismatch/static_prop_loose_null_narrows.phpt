===description===
`self::$prop == null` / `!= null` (loose) narrow a static property, the
loose-comparison counterpart of the already-existing strict `===`/`!==`
static-property null narrowing.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
class Config {
    /** @var string|null */
    public static $name = null;

    public static function falseBranchRemovesNull(): void {
        if (self::$name != null) {
            /** @mir-check self::$name is string */
            $_ = self::$name;
        }
    }

    public static function reversedOperandOrder(): void {
        if (null != self::$name) {
            /** @mir-check self::$name is string */
            $_ = self::$name;
        }
    }

    public static function trueBranchNarrowsToFalsy(): void {
        if (self::$name == null) {
            /** @mir-check self::$name is ''|'0'|null */
            $_ = self::$name;
        }
    }
}
===expect===
