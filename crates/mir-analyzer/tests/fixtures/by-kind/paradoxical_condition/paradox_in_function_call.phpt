===description===
Paradox in function call
===ignore===
TODO
===file===
<?php
/** @return 1|2|3 */
function foo() {
    /** @var 1|2|3 $bar */
    $bar = rand(1, 3);
    return $bar;
}

switch(foo()) {
    case 1: break;
    case 2: break;
    case 3: break;
    default:
        echo "bar";
}
===expect===
