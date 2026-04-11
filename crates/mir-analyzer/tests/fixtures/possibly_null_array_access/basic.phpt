===source===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    echo $arr[0];
}
===expect===
PossiblyNullArrayAccess: $arr[0]
