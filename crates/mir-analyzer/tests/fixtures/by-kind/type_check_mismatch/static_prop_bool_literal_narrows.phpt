===description===
`self::$prop === true` / `=== false` narrow a static property to the
matching bool literal (true branch) or exclude it (false branch), the
bool-literal counterpart of the already-existing static-property
int/string literal narrowing.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
class Flags {
    public static bool $enabled = false;

    public static function checkTrueBranch(): void {
        if (self::$enabled === true) {
            /** @mir-check self::$enabled is true */
            $_ = self::$enabled;
        }
    }

    public static function checkFalseBranch(): void {
        if (self::$enabled === false) {
            /** @mir-check self::$enabled is false */
            $_ = self::$enabled;
        }
    }

    public static function checkReversedOperandOrder(): void {
        if (true === self::$enabled) {
            /** @mir-check self::$enabled is true */
            $_ = self::$enabled;
        }
    }

    public static function checkNegatedFalseBranch(): void {
        if (self::$enabled !== true) {
            /** @mir-check self::$enabled is false */
            $_ = self::$enabled;
        }
    }
}
===expect===
