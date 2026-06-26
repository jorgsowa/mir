===description===
Throw on the null branch narrows the fallthrough to non-null — no PossiblyNullArrayAccess after the guard
===file===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    if ($arr === null) {
        throw new \InvalidArgumentException('arr required');
    }
    echo $arr[0];
}
===expect===
