===description===
`$this` resolves to the enclosing class.
===cursor===
symbol
===file===
<?php
final class Counter {
    public function self(): self { return $th<CURSOR>is; }
}
===expect===
kind: variable $this
type: Counter
