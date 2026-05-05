===description===
foreachNonEmptyMultipleTypes
===file===
<?php
$items = [1]; // Non-empty literal array

$result = null; // Initial type: null
foreach ($items as $i) {
    if ($i > 0.5) {
        $result = "string";
    } else {
        $result = 42;
    }
}

// After loop, result type still includes null since it was pre-initialized
// This documents the current behavior - unconditional assignment detection
// would require more complex control flow analysis
if (is_string($result) || is_int($result)) {
    echo "valid";
}
===expect===
