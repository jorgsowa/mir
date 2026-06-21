===description===
FP-G (known parser bug): `10 ** $this->maxDigits` is mis-parsed as
`(10 ** $this)->maxDigits` by php-rs-parser 0.18.0 because `**` and `->`
are given wrong precedence. The three diagnostics below are all false
positives that vanish once the parser is fixed (tracked upstream).
===config===
php_version=8.2
===file===
<?php

class Formatter {
    private int $maxDigits = 6;

    public function max(): float {
        return 10 ** $this->maxDigits;
    }
}
===expect===
InvalidOperand@7:15-7:26: Operator '**' not supported between '10' and 'Formatter'
InvalidPropertyFetch@7:15-7:37: Cannot fetch property on non-object type 'int|float'
MixedReturnStatement@7:8-7:38: Cannot return a mixed type from function with declared return type 'float'

