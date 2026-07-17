<?php
namespace Psalm;

use Psalm\Type\Union;

abstract class Type
{
    public static function parseString(string $type_string): Union
    {
        return new Union($type_string);
    }

    public static function getMixed(): Union
    {
        return new Union('mixed');
    }
}
