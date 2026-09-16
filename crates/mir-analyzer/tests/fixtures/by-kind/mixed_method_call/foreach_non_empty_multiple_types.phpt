===description===
Foreach non empty multiple types
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

// A non-empty literal guarantees one assignment, so null is eliminated.
/** @mir-check $result is string|int */
if (is_string($result) || is_int($result)) {
    echo "valid";
}
===expect===
RedundantCondition@15:4-15:41: Condition is always true/false for type 'bool'
