===description===
Literal bool/int/string `===` comparisons narrow property receivers, same
as they already do for plain variables — extract_prop_access was missing
from these three arms even though the null/enum-case arms already support
it.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingPropertyType
===file===
<?php

class Holder {
    public bool|string $flag = true;
    /** @var 1|2|3 */
    public $count = 1;
    /** @var 'active'|'inactive'|'pending' */
    public $status = 'active';
}

function narrowsBoolTrue(Holder $h): void {
    if ($h->flag === true) {
        $x = $h->flag;
        /** @mir-check $x is true */
        $_ = $x;
    }
}

function narrowsBoolFalseSymmetric(Holder $h): void {
    if (false === $h->flag) {
        $x = $h->flag;
        /** @mir-check $x is false */
        $_ = $x;
    }
}

function narrowsIntLiteral(Holder $h): void {
    if ($h->count === 2) {
        $x = $h->count;
        /** @mir-check $x is 2 */
        $_ = $x;
    }
}

function narrowsIntLiteralSymmetric(Holder $h): void {
    if (2 === $h->count) {
        $x = $h->count;
        /** @mir-check $x is 2 */
        $_ = $x;
    }
}

function narrowsStringLiteral(Holder $h): void {
    if ($h->status === 'active') {
        $x = $h->status;
        /** @mir-check $x is 'active' */
        $_ = $x;
    }
}

function narrowsStringLiteralSymmetric(Holder $h): void {
    if ('active' === $h->status) {
        $x = $h->status;
        /** @mir-check $x is 'active' */
        $_ = $x;
    }
}
===expect===
