===description===
Overwrite ordered named param
===file===
<?php
function test(int $param, int $param2): void {
    echo $param + $param2;
}

test(1, param: 2);
===expect===
InvalidNamedArgument@6:9-6:17: test() has no parameter named $param
