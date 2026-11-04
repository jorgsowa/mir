===description===
floor()'s integral float, negated or not, fits an int or ?int parameter
===file===
<?php
function f(string $p, int $n): string {
    $limit = floor($n / 2);
    return substr($p, 0, $limit) . substr($p, -$limit);
}
