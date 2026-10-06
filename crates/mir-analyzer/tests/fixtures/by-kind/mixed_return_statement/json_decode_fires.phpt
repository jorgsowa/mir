===description===
json_decode() returns mixed; MixedReturnStatement fires when declared return is a concrete type
===file===
<?php
function decode(): string {
    return json_decode('{"key":"value"}');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
}
