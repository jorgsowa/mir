===description===
Negative control for the nullable-RHS property-narrowing fix: when the
non-null part of the assigned type is NOT a subtype of the declared property
type, the property must NOT narrow — the read still resolves against the
declared type, so a genuinely undefined method stays flagged.
===file===
<?php
class Base {}
class Other {}
class Finder {
    /** @return Other|null */
    public function find() {
        return new Other();
    }
}
class Widget {
    /** @var Base */
    protected $item;
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
MissingPropertyType@12:4-12:19: Property Widget::$item has no type annotation
InvalidPropertyAssignment@19:8-19:43: Property $item expects 'Base', cannot assign 'Other|null'
UndefinedMethod@20:8-20:28: Method Base::extra() does not exist
