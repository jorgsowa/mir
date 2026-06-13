===description===
Empty array unions fold into generic arrays - comprehensive cases
===config===
suppress=MixedAssignment,UnusedForeachValue
===file===
<?php

/**
 * @return array<int, string>
 */
function loopMayNotExecute(): array
{
    $result = [];
    foreach ([] as $item) {
        $result[1] = "value";
    }
    return $result;
}

/**
 * @return array<string, bool>
 */
function conditionalAssignment(bool $flag): array
{
    $out = [];
    if ($flag) {
        $out["key"] = true;
    }
    return $out;
}

/**
 * @return array<int, int>
 */
function multipleBranches(bool $flag): array
{
    $data = [];
    if ($flag) {
        $data[1] = 100;
    } else {
        $data[2] = 200;
    }
    return $data;
}
===expect===
