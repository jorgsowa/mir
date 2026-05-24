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
UndefinedClass@3:13: Class ExampleUnknownA does not exist
UndefinedClass@3:29: Class ExampleUnknownB does not exist
InvalidPropertyAssignment@5:9: Property $other expects 'ExampleUnknownA&ExampleUnknownB', cannot assign 'ExampleUnknownAB'
UndefinedClass@5:28: Class ExampleUnknownAB does not exist
