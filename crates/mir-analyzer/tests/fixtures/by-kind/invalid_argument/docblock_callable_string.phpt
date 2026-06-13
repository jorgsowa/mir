===description===
Docblock callable string
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @param callable-string $callback
 */
function executeCallback($callback) {
    return $callback();
}

function myFunction() {
    return "result";
}

// This SHOULD be resolvable because it came from documented callable type
executeCallback("myFunction");
===expect===
