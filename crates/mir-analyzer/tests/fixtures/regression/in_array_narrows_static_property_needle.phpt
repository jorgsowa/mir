===description===
`in_array($needle, [...])` narrows a static-property needle like its
var/instance-property counterparts already do, both true and false
branches.
===config===
suppress=MissingConstructor,UnusedParam,UnusedVariable
===file===
<?php
class Container {
    /** @var 'a'|'b'|'c'|'d' */
    private static string $status = 'a';

    // Positive (true branch): narrows away 'd'.
    public static function testTrue(): void {
        if (in_array(self::$status, ['a', 'b', 'c'], true)) {
            /** @mir-check self::$status is 'a'|'b'|'c' */
            $_ = self::$status;
        }
    }

    // Positive (false branch): excludes the matched literals, leaving 'a'.
    public static function testFalse(): void {
        if (!in_array(self::$status, ['b', 'c', 'd'], true)) {
            /** @mir-check self::$status is 'a' */
            $_ = self::$status;
        }
    }
}
===expect===
