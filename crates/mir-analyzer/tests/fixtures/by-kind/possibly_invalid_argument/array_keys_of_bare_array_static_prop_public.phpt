===description===
Public static arrays do not retain key types inferred from literal defaults.
===config===
suppress=ForbiddenCode
===file===
<?php
class PublicHolder
{
    /** @var array */
    public static $items = ['a' => 1, 'b' => 2];

    public static function keys(): void
    {
        foreach (array_keys(self::$items) as $key) {
            takesInt($key);
//                   ^^^^ PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|string' provided
        }
    }
}

function takesInt(int $n): void
{
    var_dump($n);
}
===expect===
