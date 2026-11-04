===description===
A foreach over a non-empty list still enters its body with the pre-loop state
===file===
<?php
/** @param non-empty-list<string> $xs */
function first(array $xs): ?string {
    $found = null;
    foreach ($xs as $x) {
        if (null === $found) {
            $found = $x;
        }
    }
    return $found;
}
