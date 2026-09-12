===description===
`associative-array` accepts matching string-keyed return values.
===file===
<?php
/**
 * @return associative-array<string, int>
 */
function buildAssociativeArray() {
    return ['x' => 1, 'y' => 2];
}
===expect===
