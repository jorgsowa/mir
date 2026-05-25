===description===
Undefined function in docblock
===file===
<?php
/**
 * @param callable-string $callback
 */
function executeCallback($callback) {
    return $callback();
}

// Passing a non-existent function reference in docblock context
// SHOULD emit UndefinedFunction because it's documented as callable
executeCallback("nonExistentFunction");
===expect===
UndefinedFunction@11:17: Function nonExistentFunction() is not defined
