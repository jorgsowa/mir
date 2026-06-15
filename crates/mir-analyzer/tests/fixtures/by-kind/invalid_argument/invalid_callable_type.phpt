===description===
Invalid callable type
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @param callable $callback
 */
function executeCallback($callback) {
    return $callback();
}

// Passing an array that's not a valid callable format
// SHOULD emit InvalidArgument because array is not a valid callable
executeCallback(["invalid"]);
===expect===
InvalidArgument@11:16-11:27: Argument $callback of callable() expects 'callable (string or [object, "method"])', got 'array{0: "invalid"}'
