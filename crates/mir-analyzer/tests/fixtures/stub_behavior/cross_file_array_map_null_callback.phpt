===file:Zipper.php===
<?php
function zipArrays(array $a, array $b): array {
    // null callback (zip mode) must be accepted — callable|null signature
    return array_map(null, $a, $b);
}
===file:Main.php===
<?php
$pairs = zipArrays([1, 2], ['a', 'b']);
===expect===
