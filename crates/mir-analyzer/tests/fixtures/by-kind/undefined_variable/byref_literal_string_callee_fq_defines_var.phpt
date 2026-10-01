===description===
Leading-backslash literal function name premarks by-ref out-param
===file===
<?php
function b(string $s): int {
    $fn = '\preg_match';
    $fn('/a/', $s, $m);
    return count($m);
}
===expect===
