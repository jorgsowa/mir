===description===
Mixed inferred return statement
===file===
<?php
function fooFoo(array $arr): string {
    return array_pop($arr);
}
===expect===
MixedReturnStatement@3:4-3:27: Cannot return a mixed type from function with declared return type 'string'
