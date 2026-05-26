===description===
Should warn about no generator return
===file===
<?php
function generator2() : Generator {
    if (rand(0,1)) {
        return;
    }
    yield 2;
}

/**
 * @suppress InvalidNullableReturnType
 */
function notagenerator() : Generator {
    if (rand(0, 1)) {
        return;
    }
    return generator2();
}
===expect===
InvalidReturnStatement
===ignore===
TODO
