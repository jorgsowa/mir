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
//      ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'Generator'
    }
    return generator2();
}
===expect===
UnusedSuppress@10:13-10:38: Suppress annotation for 'InvalidNullableReturnType' is never used
