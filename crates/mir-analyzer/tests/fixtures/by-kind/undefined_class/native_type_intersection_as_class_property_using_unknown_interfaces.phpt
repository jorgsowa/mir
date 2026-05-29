===description===
Native type intersection as class property using unknown interfaces
===config===
suppress=InvalidPropertyAssignment,UnusedProperty
===file===
<?php
class C {
    private ExampleUnknownA&ExampleUnknownB $other;
    public function __construct() {
        $this->other = new ExampleUnknownAB();
    }
}
===expect===
UndefinedClass@3:13-3:28: Class ExampleUnknownA does not exist
UndefinedClass@3:29-3:44: Class ExampleUnknownB does not exist
UndefinedClass@5:28-5:44: Class ExampleUnknownAB does not exist
