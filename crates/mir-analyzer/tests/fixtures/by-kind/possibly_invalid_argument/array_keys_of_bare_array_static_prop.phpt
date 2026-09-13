===description===
Private static arrays retain key types inferred from literal defaults.
===config===
suppress=ForbiddenCode
===file===
<?php
class Cookie
{
    /** @var array */
    private static $defaults = ['Name' => null, 'Value' => null, 'Path' => '/'];

    public static function keys(): void
    {
        foreach (array_keys(self::$defaults) as $key) {
            takesString($key);
        }
    }
}

function takesString(string $s): void
{
    var_dump($s);
}
===expect===
