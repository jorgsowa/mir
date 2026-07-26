===description===
Same gap as the instance-call-syntax fix, for a method reached through
self::/static:: call syntax — an @if-this-is constraint referencing the
method's own @template was never substituted before the comparison.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @param T $value */
    public function __construct($value) {}

    /**
     * @template U
     * @if-this-is Box<U>
     * @param U $val
     */
    public function replace($val): void {
        self::checkReplace($val);
    }

    /**
     * @template U
     * @if-this-is Box<U>
     * @param U $val
     */
    public static function checkReplace($val): void {}
}

$box = new Box('hi');
$box::checkReplace(42);
===expect===
IfThisIsMismatch@27:0-27:22: Cannot call Box::checkReplace() — @if-this-is requires $this to be 'Box<U>', but it is 'Box<string>'
