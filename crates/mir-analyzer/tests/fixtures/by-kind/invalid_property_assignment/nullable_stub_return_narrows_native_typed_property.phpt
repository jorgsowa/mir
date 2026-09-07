===description===
A native-typed (non-nullable) property assigned a nullable RHS is itself an
invalid assignment (kept, real bug) — but the property must still narrow to
the assigned subtype afterward instead of falling back to the declared
supertype, which previously produced a spurious UndefinedMethod.
===file===
<?php
class Base {}
class Derived extends Base {
    public function extra(): void {}
}
class Finder {
    /** @return Derived|null */
    public function find() {
        return new Derived();
    }
}
class Widget {
    protected Base $item;
    private Finder $finder;
    public function __construct(Finder $finder) {
        $this->finder = $finder;
        $this->item = new Base();
    }
    public function load(): void {
        $this->item = $this->finder->find();
        $this->item->extra();
    }
}
===expect===
InvalidPropertyAssignment@20:8-20:43: Property $item expects 'Base', cannot assign 'Derived|null'
PossiblyNullMethodCall@21:8-21:28: Cannot call method extra() on possibly null value
