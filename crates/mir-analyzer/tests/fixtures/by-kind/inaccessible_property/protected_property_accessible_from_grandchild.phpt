===description===
InaccessibleProperty does NOT fire when a grandchild class accesses a protected property from a grandparent.
===file===
<?php
class GrandParent
{
    protected int $limit = 100;
}

class Mid extends GrandParent
{
}

class GrandChild extends Mid
{
    public function getLimit(): int
    {
        return $this->limit;
    }
}
===expect===
