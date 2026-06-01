===description===
Bare callable default with null coalesce, then called with arg - no TooManyArguments
===file===
<?php
function process(?callable $callback): void {
    $callback ??= fn () => true;
    $callback('arg');
}

===expect===
