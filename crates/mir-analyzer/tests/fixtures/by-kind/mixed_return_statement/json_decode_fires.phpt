===description===
json_decode() returns mixed; MixedReturnStatement fires when declared return is a concrete type
===file===
<?php
function decode(): string {
    return json_decode('{"key":"value"}');
}
===expect===
MixedReturnStatement@3:4-3:42: Cannot return a mixed type from function with declared return type 'string'
