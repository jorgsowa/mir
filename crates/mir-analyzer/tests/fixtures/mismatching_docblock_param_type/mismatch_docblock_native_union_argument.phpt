===description===
Mismatch docblock native union argument
===file===
<?php
/**
 * @param string|null $in
 */
function test(int|bool $in): bool {
    return !!$in;
}

===expect===
MismatchingDocblockParamType
===ignore===
TODO
