===description===
interface-string<T> argument binds T to the named interface, not the bound
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Wrapper {}

interface Shape {}
interface Polygon {}

/**
 * @template T of object
 * @param interface-string<T> $iface
 * @return Wrapper<T>
 */
function make(string $iface): Wrapper { return new Wrapper(); }
//            ^^^^^^^^^^^^^ UnusedParam: Parameter $iface is never used

$shapeWrapper = make(Shape::class);
$polygonWrapper = make(Polygon::class);
/** @mir-check $shapeWrapper is Wrapper<Shape> */
/** @mir-check $polygonWrapper is Wrapper<Polygon> */
===expect===
