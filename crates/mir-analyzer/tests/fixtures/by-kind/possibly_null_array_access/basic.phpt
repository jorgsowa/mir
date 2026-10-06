===description===
Basic
===file===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    echo $arr[0];
//       ^^^^^^^ PossiblyNullArrayAccess: Cannot access array on possibly null value
}
