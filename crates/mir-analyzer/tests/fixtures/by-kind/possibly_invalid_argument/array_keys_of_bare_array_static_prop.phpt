===description===
M7: array_keys() of a private static property declared as bare `array`
with a literal initializer. The collector refines the property type from
the literal default (string keys), so the keys are `string`, not
`int|string`, and no PossiblyInvalidArgument is reported.
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
