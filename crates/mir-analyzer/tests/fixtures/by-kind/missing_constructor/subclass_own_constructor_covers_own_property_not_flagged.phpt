===description===
Negative control. A subclass that declares its own constructor is not
flagged for its own uninitialized property, even though its ancestor also
declares an (unrelated) constructor.
===file===
<?php
abstract class TestCaseBase {
    public function __construct() {}
}
class WidgetTest extends TestCaseBase {
    private Logger $logger;
    public function __construct() {
        $this->logger = new Logger();
    }
}
class Logger {}

===expect===
