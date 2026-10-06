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
//  ^^^^^^^^^^^^^^^^^^ UndefinedClass: Class NoSuchClassOutside does not exist
===expect===
UnusedSuppress@2:20-2:34: Suppress annotation for 'UndefinedClass' is never used
