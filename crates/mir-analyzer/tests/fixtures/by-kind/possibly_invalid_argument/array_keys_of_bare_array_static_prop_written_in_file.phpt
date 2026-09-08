===description===
M7 negative: a private static bare-`array` property that is *written*
within the file (with different keys than the default) must NOT be
refined from its literal initializer, so the keys stay `int|string`
and the possibly-invalid argument is still reported.
===config===
suppress=ForbiddenCode
===file===
<?php
class Holder
{
    /** @var array */
    private static $items = ['a' => 1, 'b' => 2];

    public static function rewrite(): void
    {
        self::$items = ['x', 'y'];
    }

    public static function keys(): void
    {
        foreach (array_keys(self::$items) as $key) {
            takesString($key);
        }
    }
}

function takesString(string $s): void
{
    var_dump($s);
}
===expect===
PossiblyInvalidArgument@15:24-15:28: Argument $s of takesString() expects 'string', possibly different type 'int|string' provided
