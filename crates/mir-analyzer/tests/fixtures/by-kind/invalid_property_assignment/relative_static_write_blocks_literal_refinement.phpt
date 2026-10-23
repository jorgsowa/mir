===description===
A `namespace\Cookie::$defaults` write counts as a write to that property, so
its declared bare `array` is not narrowed to the literal default.
===file===
<?php
namespace App;

class Cookie
{
    /** @var array */
    private static $defaults = ['Name' => null, 'Value' => null];

    public static function keys(): void
    {
        foreach (array_keys(self::$defaults) as $key) {
            takesString($key);
//                      ^^^^ PossiblyInvalidArgument: Argument $s of takesString() expects 'string', possibly different type 'int|string' provided
        }
    }

    public static function reset(): void
    {
        namespace\Cookie::$defaults = [1, 2];
    }
}

function takesString(string $s): void
{
    echo $s;
}
