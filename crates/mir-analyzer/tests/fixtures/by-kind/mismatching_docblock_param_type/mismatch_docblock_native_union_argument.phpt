===description===
Mismatch docblock native union argument
===ignore===
TODO
===file===
<?php
/**
 * @param string|null $in
 */
function test(int|bool $in): bool {
    return !!$in;
}

===expect===
