===description===
mir-check works with function parameters
===file===
<?php
function process(string $input, int $count): void {
    /** @mir-check $input is string */
    /** @mir-check $count is int */
    echo $input . $count;
}
===expect===
