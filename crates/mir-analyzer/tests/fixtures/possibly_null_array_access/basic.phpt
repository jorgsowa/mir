===description===
basic
===file===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    echo $arr[0];
}
===expect===
PossiblyNullArrayAccess@6:10: Cannot access array on possibly null value
