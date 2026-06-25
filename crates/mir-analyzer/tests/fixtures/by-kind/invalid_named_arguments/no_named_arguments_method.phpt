===description===
InvalidNamedArguments fires when a named argument is passed to a @no-named-arguments method.
===file===
<?php
class Calculator {
    /**
     * @no-named-arguments
     */
    public function add(int $a, int $b): int {
        return $a + $b;
    }
}

$calc = new Calculator();
$calc->add(a: 1, b: 2);
===expect===
InvalidNamedArguments@12:11-12:15: add() does not accept named arguments
InvalidNamedArguments@12:17-12:21: add() does not accept named arguments
