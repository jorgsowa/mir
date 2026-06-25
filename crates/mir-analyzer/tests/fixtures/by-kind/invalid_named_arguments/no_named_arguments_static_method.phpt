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
===expect===
InvalidNamedArguments@11:10-11:14: max() does not accept named arguments
InvalidNamedArguments@11:16-11:20: max() does not accept named arguments
