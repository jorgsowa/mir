===description===
MissingConstructor fires when a subclass adds its own uninitialized native
property but relies entirely on an inherited constructor — that constructor
was written in the ancestor, before the subclass's property existed, so it
can't be the thing that initializes it (the classic test-framework
base-class + setUp() convention).
===file===
<?php
abstract class TestCaseBase {
    public function __construct() {}
}
class WidgetTest extends TestCaseBase {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class WidgetTest has uninitialized properties but no constructor
    private Logger $logger;
    public function setUp(): void {
        $this->logger = new Logger();
    }
}
class Logger {}
