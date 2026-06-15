===description===
SKIPPED-referenceToTypedArrayConstrainsAssignmentWithNullReferenceInitialization
===config===
suppress=UnusedVariable
===file===
<?php
class Foo
{
    /** @var list<int> */
    public array $arr = [];

    public function __construct()
    {
        $int = &$this->arr[0]; // If $this->arr[0] isn't set, this will set it to null.
    }
}

===expect===
UnsupportedReferenceUsage@9:8-9:29: Reference assignment is not supported
