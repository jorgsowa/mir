===description===
cross file multiple interfaces one missing
===file:Stringable.php===
<?php
interface HasLabel {
    public function getLabel(): string;
}
===file:Countable.php===
<?php
interface HasCount {
    public function getCount(): int;
}
===file:Entity.php===
<?php
class Entity implements HasLabel, HasCount {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Entity must implement HasCount::getCount() from interface
    public function getLabel(): string { return ""; }
    # getCount() is NOT implemented
}
===expect===
