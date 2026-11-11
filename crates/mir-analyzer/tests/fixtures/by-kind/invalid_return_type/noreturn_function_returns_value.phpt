===description===
`@return noreturn` is `never`, so returning a value is rejected.
===file===
<?php
/** @return noreturn */
function claimsNoreturn() {
    return 1;
//  ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'never'
}

/** @return noreturn */
function alwaysThrows() {
    throw new \RuntimeException('stop');
}
