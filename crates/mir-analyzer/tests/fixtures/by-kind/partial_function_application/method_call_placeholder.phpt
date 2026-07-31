===description===
A `?` placeholder in an instance method call parses to `Arg { value: None }`
the same way a plain function call does — must not crash or misbehave
differently for the method-call code path.
===config===
suppress=UnusedVariable
===file===
<?php

class Calculator {
    public function add(int $a, int $b): int {
        return $a + $b;
    }
}

$calc = new Calculator();
$partial = $calc->add(?, 5);
===expect===
ParseError@10:22-10:23: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
