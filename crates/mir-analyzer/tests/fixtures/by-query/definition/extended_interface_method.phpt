===description===
A method declared on a parent interface lands on that interface.
===cursor===
definition
===file===
<?php
interface Readable {
    public function read(): string;
}
interface Stream extends Readable {}
function f(Stream $s): string {
    return $s->re<CURSOR>ad();
}
===expect===
test.php@3:4-3:35
