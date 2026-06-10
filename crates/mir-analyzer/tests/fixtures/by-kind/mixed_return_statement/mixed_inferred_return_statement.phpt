===description===
Mixed inferred return statement
===ignore===
TODO
===file===
<?php
function fooFoo(array $arr): string {
    return array_pop($arr);
}
===expect===
