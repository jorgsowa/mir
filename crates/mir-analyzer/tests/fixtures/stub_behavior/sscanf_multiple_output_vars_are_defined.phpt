===description===
sscanf multiple output vars are defined
===file===
<?php
function parse_pair(string $input): int {
    sscanf($input, '%d %d', $left, $right);
    return $left + $right;
}
===expect===
