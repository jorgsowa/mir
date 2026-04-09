===source===
<?php
function test(bool $flag): string {
    $default = 'fallback';
    return $flag ? 'yes' : $default;
}
===expect===
