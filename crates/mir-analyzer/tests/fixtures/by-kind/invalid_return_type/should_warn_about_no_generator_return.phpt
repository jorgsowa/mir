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
UnusedPsalmSuppress@12:0-12:0: Suppress annotation for 'InvalidNullableReturnType' is never used
InvalidReturnType@14:8-14:15: Return type 'void' is not compatible with declared 'Generator'
