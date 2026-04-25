===file===
<?php
function parse(string $s): int {
    sscanf($s, '%d %d', $row, $col);
    // $row and $col are populated via variadic by-ref params — must not be UndefinedVariable
    return $row + $col;
}
===expect===
