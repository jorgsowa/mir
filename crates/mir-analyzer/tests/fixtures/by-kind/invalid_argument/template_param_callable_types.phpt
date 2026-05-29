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

/**
 * @template In
 * @template Out
 * @param callable(In): Out $transform
 */
function applyTransform(callable $transform): void {}

/**
 * @template T
 * @param callable(T, Data): T $reducer
 */
function reduce(callable $reducer): void {}

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
===expect===
UnusedParam@8:30-8:49: Parameter $processor is never used
UnusedParam@15:25-15:44: Parameter $transform is never used
UnusedParam@21:17-21:34: Parameter $reducer is never used
