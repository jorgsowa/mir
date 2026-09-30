===description===
Reported at Info severity (Psalm's PropertyNotSetInConstructor level): a property assigned in only one
branch, and a readonly property never assigned. Sibling properties that are always assigned stay clean,
and the assigned value keeps its type.
===config===
suppress=UnusedParam
===file===
<?php
class OneBranch {
    public string $value;
    public function __construct(bool $cond, string $v) {
        if ($cond) {
            $this->value = $v;
        }
        /** @mir-check $v is string */
        $v;
    }
}

class ReadonlyMissing {
    public function __construct(public readonly int $id, private readonly string $name) {}
}

class ReadonlyNeverSet {
    public readonly int $id;
    public function __construct() {}
}
===expect===
PropertyPossiblyUninitialized@4:20-4:31: Property OneBranch::$value may be left uninitialized by the constructor
PropertyPossiblyUninitialized@19:20-19:31: Property ReadonlyNeverSet::$id may be left uninitialized by the constructor
