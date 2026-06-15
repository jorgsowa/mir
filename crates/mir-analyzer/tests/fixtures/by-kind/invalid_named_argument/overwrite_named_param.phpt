===description===
Overwrite named param
===file===
<?php
function test(int $param, int $param2): void {
    echo $param + $param2;
}

test(param: 1, param: 2);
===expect===
InvalidNamedArgument@6:15-6:23: test() has no parameter named $param
