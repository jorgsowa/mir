===description===
Whitespace inside a multi-line chain resolves to the innermost enclosing call.
===cursor===
symbol
===file===
<?php
final class Builder {
    public function where(): self { return $this; }
    public function get(): int { return 1; }
}
$n = (new Builder())
    ->where()
 <CURSOR>   ->get();
===expect===
kind: method call Builder::get
type: int
