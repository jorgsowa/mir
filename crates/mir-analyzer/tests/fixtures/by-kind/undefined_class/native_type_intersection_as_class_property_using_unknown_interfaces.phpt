===description===
Native type intersection as class property using unknown interfaces
===config===
suppress=InvalidPropertyAssignment,UnusedProperty
===file===
<?php
class C {
    private ExampleUnknownA&ExampleUnknownB $other;
//          ^^^^^^^^^^^^^^^ UndefinedClass: Class ExampleUnknownA does not exist
//                          ^^^^^^^^^^^^^^^ UndefinedClass: Class ExampleUnknownB does not exist
    public function __construct() {
        $this->other = new ExampleUnknownAB();
//                         ^^^^^^^^^^^^^^^^ UndefinedClass: Class ExampleUnknownAB does not exist
    }
}
===expect===
