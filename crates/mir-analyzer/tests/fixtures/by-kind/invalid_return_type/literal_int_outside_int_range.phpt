===description===
A literal int outside the declared int range fails return-type checking.
===file===
<?php
/**
 * @return int<1, 5>
 */
function test(): int {
    return 15;
//  ^^^^^^^^^^ InvalidReturnType: Return type '15' is not compatible with declared 'int<1, 5>'
}
===expect===
