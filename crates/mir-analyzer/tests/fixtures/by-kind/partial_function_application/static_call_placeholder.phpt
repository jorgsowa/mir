===description===
Same placeholder-argument coverage as the plain-function and instance-method
fixtures, but through a static method call.
===config===
suppress=UnusedVariable
===file===
<?php

class MathUtil {
    public static function add(int $a, int $b): int {
        return $a + $b;
    }
}

$partial = MathUtil::add(?, 5);
===expect===
ParseError@9:25-9:26: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
