===description===
callable types with template parameters should not report InvalidArgument false positives
===file===
<?php
class Data { }

/**
 * @template T
 * @param callable(T): void $processor
 */
function processWithCallback(callable $processor): void {}
//                           ^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $processor is never used

/**
 * @template In
 * @template Out
 * @param callable(In): Out $transform
 */
function applyTransform(callable $transform): void {}
//                      ^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $transform is never used

/**
 * @template T
 * @param callable(T, Data): T $reducer
 */
function reduce(callable $reducer): void {}
//              ^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $reducer is never used

function test(): void {
    // Callback that accepts Data
    $fn1 = function(Data $d): void { };
    processWithCallback($fn1);

    // Callback that transforms Data to string
    $fn2 = function(Data $d): string { return 'str'; };
    applyTransform($fn2);

    // Reducer that takes Data and returns Data
    $fn3 = function(Data $acc, Data $item): Data { return $acc; };
    reduce($fn3);
}
