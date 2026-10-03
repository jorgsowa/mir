===description===
@phpstan-type alias defined on a standalone function does not produce false positives
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/**
 * @phpstan-type Point = array{x: int, y: int}
 * @param Point $p
 * @return int
 */
function sumCoords(array $p): int {
    return $p['x'] + $p['y'];
}

sumCoords(['x' => 1, 'y' => 2]);
===expect===
