===description===
Negative control. The uninitialized property is declared on the SAME class
that owns the constructor covering it; a subclass that adds no properties of
its own must not be flagged just because it also has no constructor.
===file===
<?php
class Base {
    private Logger $logger;
    public function __construct() {
        $this->logger = new Logger();
    }
}
class Sub extends Base {}
class Logger {}

===expect===
