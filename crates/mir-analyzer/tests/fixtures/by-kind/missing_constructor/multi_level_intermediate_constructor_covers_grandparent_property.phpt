===description===
Negative control, three-level hierarchy. The intermediate class's own
constructor is closer to (more derived than) the grandparent property it
inherits, so it can legitimately be the thing that initializes it — the
leaf class must not be flagged just for adding no constructor of its own.
===file===
<?php
abstract class GrandBase {
    private Logger $logger;
}
class Middle extends GrandBase {
    public function __construct() {
        $this->logger = new Logger();
    }
}
class Leaf extends Middle {}
class Logger {}

===expect===
