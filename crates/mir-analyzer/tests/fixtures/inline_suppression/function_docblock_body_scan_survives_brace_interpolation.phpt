===description===
A `"{$expr}"` string-interpolation brace pair inside the body is balanced
and must not throw off the body-extension scan's depth tracking — the
diagnostic right after the function is still reported.
===file===
<?php
/** @psalm-suppress UndefinedClass */
function f(): void {
    $name = "world";
    echo "hello {$name}";
}
new NoSuchClassOutside();
===expect===
UnusedSuppress@3:0-3:0: Suppress annotation for 'UndefinedClass' is never used
UndefinedClass@7:4-7:22: Class NoSuchClassOutside does not exist
