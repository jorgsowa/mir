===description===
`@psalm-assert-if-true` used as an `if` condition on a NULLSAFE method-call
receiver ($v?->isPositive($x)) never narrowed at all -- the narrowing
dispatch had arms for MethodCall and StaticMethodCall but no
NullsafeMethodCall, unlike taint.rs's taint-source check which already
pairs both.
===config===
suppress=UnusedVariable,UnusedParam,MissingParamType
===file===
<?php
class Validator {
    /** @psalm-assert-if-true int $value */
    public function isPositive($value): bool {
        return true;
    }
}

function f(?Validator $v, $x): void {
    if ($v?->isPositive($x)) {
        /** @mir-check $x is int */
        $_ = 1;
    }
}
===expect===
