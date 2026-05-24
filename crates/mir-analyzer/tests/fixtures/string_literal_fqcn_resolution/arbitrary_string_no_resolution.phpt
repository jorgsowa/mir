===description===
arbitraryStringNoResolution
===file===
<?php
// A plain string literal that happens to match a function name
// should NOT be resolved as a callable or cause UndefinedClass errors

$callback = "array_map";  // Looks like a function name but it's just a string
array_map("trim", ["  hello  ", "  world  "]);

// Even arbitrary string values in callback position should not emit false positives
function processArray($data) {
    // Using a known function string - should NOT emit false positives
    return array_filter($data, "strlen");  // strlen exists, so this is OK
}
===expect===
