===description===
Mixed inferred return statement
===file===
<?php
function fooFoo(array $arr): string {
    return array_pop($arr);
}
===expect===
MixedReturnStatement
===ignore===
TODO
