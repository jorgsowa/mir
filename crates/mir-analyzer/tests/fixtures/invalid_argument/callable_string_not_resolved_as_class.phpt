===description===
callableStringNotResolvedAsClass
===file===
<?php
// A function name passed as string should NOT emit UndefinedClass
// even if "processData" sounds like it could be a class name

/**
 * @param callable $callback
 */
function execute(callable $callback) {
    $callback();
}

function processData() {
    return "data";
}

// This should NOT emit UndefinedClass for "processData"
execute("processData");
===expect===
