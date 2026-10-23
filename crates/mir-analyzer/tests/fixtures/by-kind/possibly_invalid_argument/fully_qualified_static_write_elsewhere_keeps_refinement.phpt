===description===
A write to another class's static property through a fully qualified
single-segment name (`\Counter::$n`) is resolved, so it does not keep
`Cookie::$defaults` from being typed by its literal default.
===file===
<?php
class Cookie
{
    /** @var array */
    private static $defaults = ['Name' => null, 'Value' => null];

    public static function keys(): void
    {
        foreach (array_keys(self::$defaults) as $key) {
            takesString($key);
        }
    }
}

class Counter
{
    public static int $n = 0;
}

function bump(): void
{
    \Counter::$n = 1;
}

function takesString(string $s): void
{
    echo $s;
}
