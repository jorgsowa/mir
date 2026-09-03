===description===
A nullable RHS (e.g. a stub/mock method typed `T|null`) assigned to a
property declared as a wider supertype must still narrow the property to `T`
(plus null) — the null part alone shouldn't discard the whole narrowing.
Covers both instance and static properties.
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
    /** @return Derived|null */
    public static function findStatic() {
        return new Derived();
    }
}
class Widget {
    /** @var Base */
    protected $item;
    /** @var Base */
    protected static $staticItem;
    private Finder $finder;
    public function __construct(Finder $finder) {
        $this->finder = $finder;
    }
    public function load(): void {
        $this->item = $this->finder->find();
        // Narrowed to Derived|null, not the declared Base — a possibly-null
        // call, not an undefined method on Base.
        $this->item->extra();
    }
    public static function loadStatic(): void {
        self::$staticItem = Finder::findStatic();
        self::$staticItem->extra();
    }
}
===expect===
PossiblyNullMethodCall@29:8-29:28: Cannot call method extra() on possibly null value
PossiblyNullMethodCall@33:8-33:34: Cannot call method extra() on possibly null value
