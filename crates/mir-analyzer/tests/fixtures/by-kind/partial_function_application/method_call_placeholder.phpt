===description===
A `?` placeholder in an instance method call parses to `Arg { value: None }`
the same way a plain function call does — must not crash or misbehave
differently for the method-call code path.
===config===
php_version=8.5
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
//                    ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
===expect===
