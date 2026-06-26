===description===
is_array() in the if-condition narrows away the null atom — no PossiblyNullArrayAccess inside the branch
===file===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    if (is_array($arr)) {
        echo $arr[0];
    }
}
===expect===
