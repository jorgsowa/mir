===description===
Callable arity mismatch
===config===
suppress=MissingParamType,MissingReturnType,MixedArgument
===file===
<?php
// Function with wrong arity
function processItem($item, $extra) {
    return strtoupper($item) . $extra;
}

// SHOULD emit error because array_map expects 1 param, but processItem needs 2
array_map("processItem", ["a", "b"]);
===expect===
TooFewArguments@8:10-8:23: Too few arguments for processItem(): expected 2, got 1
