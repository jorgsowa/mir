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
//         ^^^^ InvalidNamedArguments: add() does not accept named arguments
//               ^^^^ InvalidNamedArguments: add() does not accept named arguments
