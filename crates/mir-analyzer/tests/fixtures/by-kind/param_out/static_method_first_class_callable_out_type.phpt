===description===
@param-out type is preserved when a static method is captured as a
first-class callable. $out should be int (from @param-out int), not mixed.
===config===
suppress=UnusedVariable
===file===
<?php
class Counter {
    /**
     * @param-out int $n
     */
    public static function next(mixed &$n): void {
        static $i = 0;
        $n = ++$i;
    }
}

$fn = Counter::next(...);
$fn($out);
/** @mir-check $out is int */
echo $out;
===expect===
