===description===
A templated method's return type is inferred from its argument.
===cursor===
symbol
===file===
<?php
final class Box {
    /**
     * @template T
     * @param T $value
     * @return T
     */
    public function wrap(mixed $value): mixed { return $value; }
}
$n = (new Box())->wr<CURSOR>ap(42);
===expect===
kind: method call Box::wrap
type: 42
