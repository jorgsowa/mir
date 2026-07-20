===description===
`in_array($x, self::$ALLOWED, true)` narrows `$x` when the haystack is a
static-property array, mirroring the literal-array and instance-property
haystack cases already supported.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Container {
    /** @var array{0: 'a', 1: 'b', 2: 'c'} */
    private static array $allowed = ['a', 'b', 'c'];

    public static function test(string $x): string {
        if (in_array($x, self::$allowed, true)) {
            /** @mir-check $x is 'a'|'b'|'c' */
            $_ = $x;
            return $x;
        }
        return '';
    }
}
===expect===
