===description===
`throw throw $e` throws the inner exception; the outer operand is `never`.
===file===
<?php
function f(): never {
    throw throw new RuntimeException('x');
}
