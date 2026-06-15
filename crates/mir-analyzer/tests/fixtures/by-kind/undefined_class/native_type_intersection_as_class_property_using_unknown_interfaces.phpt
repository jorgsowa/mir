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
UndefinedClass@3:12-3:27: Class ExampleUnknownA does not exist
UndefinedClass@3:28-3:43: Class ExampleUnknownB does not exist
UndefinedClass@5:27-5:43: Class ExampleUnknownAB does not exist
