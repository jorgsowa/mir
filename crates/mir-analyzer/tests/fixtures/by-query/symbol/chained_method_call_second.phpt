===description===
The second call of a chain resolves against the first call's return type.
===cursor===
symbol
===file===
<?php
final class Builder {
    public function where(): self { return $this; }
    public function get(): int { return 1; }
}
$n = (new Builder())->where()->g<CURSOR>et();
===expect===
kind: method call Builder::get
type: int
