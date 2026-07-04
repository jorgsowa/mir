===description===
`float <-> int` between a typed-callable's documented parameter and the actual
closure's declared parameter must not be flagged. int->float is a lossless
widening; float->int is, at worst, a deprecation notice in PHP (never a
TypeError), matching the existing `ImplicitFloatToIntCast` leniency used for
direct argument checks.
===file===
<?php
/** @param callable(int):void $c1 */
function processInt(callable $c1): void {
    $c1(5);
}
processInt(function (float $a): void {});

/** @param callable(float):void $c2 */
function processFloat(callable $c2): void {
    $c2(5.5);
}
processFloat(function (int $a): void {});
===expect===
