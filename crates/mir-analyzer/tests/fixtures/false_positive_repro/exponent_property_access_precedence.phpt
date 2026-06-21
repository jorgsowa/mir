===description===
FP-G (regression): `10 ** $this->maxDigits` must parse as `10 ** ($this->maxDigits)`.
Fixed in php-rs-parser 0.18.1 (MEMBER_ACCESS_BP raised above **).
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

