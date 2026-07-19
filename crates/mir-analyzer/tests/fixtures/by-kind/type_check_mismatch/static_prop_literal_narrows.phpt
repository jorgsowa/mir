===description===
`self::$prop === 'literal'` / `=== 42` narrow a static property to the
literal (true branch) or exclude it (false branch), same as the instance-
property `$this->prop` counterpart already does — only the null-check and
instanceof narrowing families existed for static properties.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
class Config {
    public static string $env = 'prod';
    public static int $level = 1;

    public static function checkStringTrueBranch(): void {
        if (self::$env === 'staging') {
            /** @mir-check self::$env is 'staging' */
            $_ = self::$env;
        }
    }

    public static function checkStringReversedOperandOrder(): void {
        if ('staging' === self::$env) {
            /** @mir-check self::$env is 'staging' */
            $_ = self::$env;
        }
    }

    public static function checkIntTrueBranch(): void {
        if (self::$level === 2) {
            /** @mir-check self::$level is 2 */
            $_ = self::$level;
        }
    }
}

class Bounded {
    /** @var 1|2|3 */
    public static int $value = 1;

    public static function excludesLiteralOnFalseBranch(): void {
        if (self::$value !== 1) {
            /** @mir-check self::$value is 2|3 */
            $_ = self::$value;
        }
    }
}
===expect===
