===description===
callableArityMismatch
===file===
<?php
// Function with wrong arity
function processItem($item, $extra) {
    return strtoupper($item) . $extra;
}

// SHOULD emit error because array_map expects 1 param, but processItem needs 2
array_map("processItem", ["a", "b"]);
===expect===
TooFewArguments@8:11: Too few arguments for processItem(): expected 2, got 1
