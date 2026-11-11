===description===
Inside the function a `properties-of<Cls>` parameter is the shape, so its entries are typed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Point {
    public int $x = 0;
    public int $y = 0;
    public ?string $label = null;
}

function takesInt(int $value): void {}

/** @param properties-of<Point> $point */
function area(array $point): int {
    /** @mir-check $point is array{'x': int, 'y': int, 'label': string|null} */
    takesInt($point['x']);
    takesInt($point['label']);
//           ^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'string|null'
//           ^^^^^^^^^^^^^^^ PossiblyNullArgument: Argument $value of takesInt() might be null
    return $point['x'] * $point['y'];
}
