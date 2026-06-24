===description===
Passing an undefined variable to a @param-out by-ref parameter defines it —
UndefinedVariable must not fire. The out-type is the variable's type after.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @param-out array<string, mixed> $data
 */
function loadData(mixed &$data): void {
    $data = ["key" => "value"];
}

loadData($result);
/** @mir-check $result is array<string, mixed> */
$_ = $result;
===expect===
