===description===
`!($x instanceof C)` must drop a union member that's an intersection A&B when
either A or B alone would satisfy `instanceof C` — such a value is
necessarily a C too, so it can't survive the negation, the same way the
positive branch already keeps an intersection's other parts instead of
dropping them.
===config===
suppress=UnusedParam
===file===
<?php
interface Cnt3 {
    public function count3(): int;
}
interface Iter3 {
    public function next3(): void;
}
class OnlyC3 {
    public function onlyC(): void {}
}

/** @param (Cnt3&Iter3)|OnlyC3 $x */
function f($x): void {
    if (!($x instanceof Iter3)) {
        /** @mir-check $x is OnlyC3 */
        $_ = 1;
    }
}
===expect===
