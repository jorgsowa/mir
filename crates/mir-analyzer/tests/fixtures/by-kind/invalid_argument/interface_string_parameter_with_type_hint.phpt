===description===
interface-string parameter accepts a matching interface reference (positive case)
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Shape {
    public function area(): float;
}

/**
 * @param interface-string<Shape> $ifaceName
 */
function describe(string $ifaceName) {
    return $ifaceName;
}

describe(Shape::class);
===expect===
