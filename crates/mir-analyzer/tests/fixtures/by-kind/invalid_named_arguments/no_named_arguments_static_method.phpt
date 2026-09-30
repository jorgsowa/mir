===description===
InvalidNamedArguments fires when a named argument is passed to a @no-named-arguments static method.
===file===
<?php
class Math {
    /**
     * @no-named-arguments
     */
    public static function max(int $a, int $b): int {
        return $a > $b ? $a : $b;
    }
}

Math::max(a: 5, b: 3);
//        ^^^^ InvalidNamedArguments: max() does not accept named arguments
//              ^^^^ InvalidNamedArguments: max() does not accept named arguments
===expect===
