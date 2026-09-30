===description===
Go-to-definition on an abstract trait method's declaration name resolves to the declaration itself.
===cursor===
definition
===file===
<?php
trait Sizable {
    abstract public function si<CURSOR>ze(): int;
}
===expect===
test.php@3:4-3:41
