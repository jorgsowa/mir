===description===
Overwrite named param
===file===
<?php
function test(int $param, int $param2): void {
    echo $param + $param2;
}

test(param: 1, param: 2);
//             ^^^^^^^^ InvalidNamedArgument: test() argument $param overwrites a previous argument
