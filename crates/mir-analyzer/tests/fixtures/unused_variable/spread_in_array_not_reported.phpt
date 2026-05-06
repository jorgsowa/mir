===description===
spread in array not reported
===file===
<?php
function foo(array $extra): array {
    $base = [1, 2, 3];
    return [...$base, ...$extra];
}
===expect===
