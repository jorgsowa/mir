===description===
`array_keys(self::$defaults)` preserves string keys for private static defaults.
===config===
suppress=ForbiddenCode
===file===
<?php
class Cookie
{
    /** @var array */
    private static $defaults = ['Name' => null, 'Value' => null, 'Path' => '/'];

    public static function find(string $search): void
    {
        foreach (array_keys(self::$defaults) as $search) {
            takesString($search);
        }
    }
}

function takesString(string $s): void
{
    var_dump($s);
}
===expect===
