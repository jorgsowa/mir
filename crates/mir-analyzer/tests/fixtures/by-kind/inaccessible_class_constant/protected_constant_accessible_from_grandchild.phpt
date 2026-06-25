===description===
InaccessibleClassConstant does NOT fire when a grandchild class accesses a protected constant from a grandparent.
===config===
suppress=UnusedVariable
===file===
<?php
class GrandParent {
    protected const LIMIT = 100;
}

class Mid extends GrandParent {}

class GrandChild extends Mid {
    public function getLimit(): int {
        return GrandParent::LIMIT;
    }
}
===expect===
