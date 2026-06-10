===description===
Overwrite ordered with unpacked named param
===file===
<?php
function test(int $param, int $param2): void {
    echo $param + $param2;
}

test(1, ...["param" => 2]);
===expect===
