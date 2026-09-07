===description===
FP-J1: refined string atoms (non-empty-string, numeric-string,
class-string, ...) were not recognized as subtypes of `scalar` — only the
int family and TLiteralString had a `TScalar` arm in atomic_subtype. Every
one of these is still just a `string`, hence a `scalar`, at runtime.
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php

/** @param scalar $v */
function acceptsScalar($v): void {}

function nonEmpty(string $s): void {
    if ($s === '') {
        return;
    }
    acceptsScalar($s); // $s is now non-empty-string
}

function numeric(string $s): void {
    if (!is_numeric($s)) {
        return;
    }
    acceptsScalar($s); // $s is now numeric-string
}

class Widget {}

function classString(): void {
    acceptsScalar(Widget::class); // class-string
}

function fromDate(): void {
    acceptsScalar(date('Y-m-d')); // non-empty-string, the exact repro
}
===expect===
