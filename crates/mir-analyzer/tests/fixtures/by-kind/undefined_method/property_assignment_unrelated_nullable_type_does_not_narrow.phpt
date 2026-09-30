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
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $item expects 'Base', cannot assign 'Other|null'
        $this->item->extra();
//      ^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Base::extra() does not exist
    }
}
===expect===
