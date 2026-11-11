===description===
When only one member of a union argument lacks the shape's property the argument is possibly invalid.
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
final class Circle {
    public float $radius = 1.0;
}

final class Square {
    public float $side = 1.0;
}

/** @param object{radius: float}|null $shape */
function area(?object $shape): void {}

function run(Circle|Square $either, Circle $circle): void {
    area($either);
//       ^^^^^^^ PossiblyInvalidArgument: Argument $shape of area() expects 'object{radius: float}|null', possibly different type 'Circle|Square' provided
    area($circle);
    area(null);
}
