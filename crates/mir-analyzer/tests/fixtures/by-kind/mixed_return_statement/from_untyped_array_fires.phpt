===description===
Accessing an element of an untyped array returns mixed; MixedReturnStatement fires when declared return is concrete
===file===
<?php
function firstElement(array $arr): string {
    return $arr[0];
}
===expect===
MixedReturnStatement@3:4-3:19: Cannot return a mixed type from function with declared return type 'string'
