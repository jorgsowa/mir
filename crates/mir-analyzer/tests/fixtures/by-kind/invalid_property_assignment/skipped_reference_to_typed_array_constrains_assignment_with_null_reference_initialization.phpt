===description===
SKIPPED-referenceToTypedArrayConstrainsAssignmentWithNullReferenceInitialization
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
