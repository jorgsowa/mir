===description===
Undefined method in callable array
===config===
suppress=MissingReturnType
===file===
<?php
class Handler {
    public function handle() {
        return "handled";
    }
}

/**
 * @param callable $callback
 */
function executeCallback($callback) {
    return $callback();
}

$handler = new Handler();

// Using valid class with undefined method in callable array
// SHOULD emit UndefinedMethod because the method doesn't exist
executeCallback([$handler, "nonExistentMethod"]);
===expect===
UndefinedMethod@19:16-19:47: Method Handler::nonExistentMethod() does not exist
