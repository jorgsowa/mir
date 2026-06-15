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
MismatchingDocblockParamType@5:23-5:26: Docblock type 'string|null' for $in does not match inferred 'int|bool'
