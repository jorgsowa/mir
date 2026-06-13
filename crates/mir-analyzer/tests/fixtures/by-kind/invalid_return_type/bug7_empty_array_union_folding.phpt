===description===
Empty array unions fold into generic arrays (Bug 7 secondary)
===config===
suppress=MixedAssignment,UnusedForeachValue
===file===
<?php

/**
 * @return array<int, string>
 */
function mayNotExecute(): array
{
    $result = [];
    foreach ([] as $item) {
        $result[1] = "test";
    }
    return $result;
}

/**
 * @return array<string, int>
 */
function guarded(): array
{
    $out = [];
    if (false) {
        $out["key"] = 42;
    }
    return $out;
}
===expect===
