===description===
try catch error
===file===
<?php
function foo(): string {
    try {
        $result = strtolower("hi");
    } catch (\Exception $e) {
        // does not assign $result
    }
    return $result;
}
===expect===
PossiblyUndefinedVariable@8:12: Variable $result might not be defined
