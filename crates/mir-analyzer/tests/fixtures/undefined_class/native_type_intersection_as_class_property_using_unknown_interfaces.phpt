===description===
nativeTypeIntersectionAsClassPropertyUsingUnknownInterfaces
===file===
<?php
class C {
    private ExampleUnknownA&ExampleUnknownB $other;
    public function __construct() {
        $this->other = new ExampleUnknownAB();
    }
}
===expect===
UndefinedClass@3:12: Class ExampleUnknownA does not exist
UndefinedClass@3:28: Class ExampleUnknownB does not exist
InvalidPropertyAssignment@5:8: Property $other expects 'ExampleUnknownA&ExampleUnknownB', cannot assign 'ExampleUnknownAB'
UndefinedClass@5:27: Class ExampleUnknownAB does not exist
