===description===
Mismatch docblock native union argument
===file===
<?php
/**
 * @param string|null $in
 */
function test(int|bool $in): bool {
//                     ^^^ MismatchingDocblockParamType: Docblock type 'string|null' for $in does not match inferred 'int|bool'
    return !!$in;
}

===expect===
