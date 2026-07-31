===description===
Static mixed type coercion. `A[]` widens to `array<int|string, A>` (J8
fix) so assigning it to a strictly `array<int, A>`-typed property is a
genuine mismatch — was masked entirely before that fix hardcoded `int`
as the key.
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var array<int, A> */
    public static $foo = [];

    /** @param A[] $arr */
    public static function barBar(array $arr): void
    {
        self::$foo = $arr;
    }
}
===expect===
InvalidPropertyAssignment@9:8-9:25: Property $foo expects 'array<int, A>', cannot assign 'array<int|string, A>'
