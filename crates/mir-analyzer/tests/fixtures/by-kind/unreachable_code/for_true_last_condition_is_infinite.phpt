===description===
Only the last comma-separated for condition decides termination.
===file===
<?php
function a(int $n, int $limit): int {
    for ($i = 0; $i < $n, true; ++$i) {
        if ($i > $limit) {
            return $i;
        }
    }
}
function b(int $n, int $limit): int {
//                              ^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
    for ($i = 0; true, $i < $n; ++$i) {
        if ($i > $limit) {
            return $i;
        }
    }
}
