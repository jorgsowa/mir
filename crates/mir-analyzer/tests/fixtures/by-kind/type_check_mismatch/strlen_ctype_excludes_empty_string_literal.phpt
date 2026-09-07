===description===
`ctype_digit()`/`strlen($x) > 0`-style true-branch narrowing
(narrow_string_to_non_empty) rewrote the generic `string` atom to
`non-empty-string` but let a `TLiteralString("")` pass through
unchanged — a genuine false positive, since `ctype_digit('')` is false in
PHP. The mirror `strlen($x) === 0` case (narrow_string_to_empty) had the
opposite gap: it only dropped `non-empty-string`, leaving non-empty
literals/numeric-string/class-string untouched even though none of those
can ever be `""`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param '123'|'' $s */
function ctypeDigitExcludesEmptyLiteral($s): void {
    if (ctype_digit($s)) {
        /** @mir-check $s is '123' */
        $_ = 1;
    }
}

/** @param non-empty-string|'' $s */
function strlenGreaterThanZeroExcludesEmptyLiteral($s): void {
    if (strlen($s) > 0) {
        /** @mir-check $s is non-empty-string */
        $_ = 1;
    }
}

/** @param 'abc'|''|numeric-string $s */
function strlenEqualsZeroExcludesNonEmptyAtoms($s): void {
    if (strlen($s) === 0) {
        /** @mir-check $s is '' */
        $_ = 1;
    }
}
===expect===
