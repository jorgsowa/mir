===description===
A docblock intersection of scalar types (`non-empty-string&lowercase-string`,
used by Laravel's `Str::lower()`) is its narrowest part, so it fits `string`
and a string literal fits it; an intersection of objects stays one.
===file===
<?php
interface A {}
interface B {}

final class Str
{
    /**
     * @param string $value
     * @return ($value is '' ? '' : non-empty-string&lowercase-string)
     */
    public static function lower($value)
    {
        return mb_strtolower($value, 'UTF-8');
    }

    public static function snake(string $value): string
    {
        $lower = self::lower($value);
        /** @trace $lower */
        return $lower;
//      ^^^^^^^^^^^^^^ Trace: Type of $lower is ""|non-empty-string
    }

    /** @return non-empty-string&lowercase-string */
    public static function literal(): string
    {
        return 'a';
    }

    /** @return positive-int&int<1, 10> */
    public static function small(): int
    {
        return 5;
    }

    /** @param A&B $ab */
    public static function both($ab): A
    {
        /** @trace $ab */
        return $ab;
//      ^^^^^^^^^^^ Trace: Type of $ab is A&B
    }
}
