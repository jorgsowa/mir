===source===
<?php
/**
 * @param array<string>|null $arr
 */
function test(?array $arr): void {
    if ($arr !== null) {
        echo $arr[0];
    }
}
===expect===
