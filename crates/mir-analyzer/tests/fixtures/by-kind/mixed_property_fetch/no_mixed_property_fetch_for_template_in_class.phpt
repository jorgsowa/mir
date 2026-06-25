===description===
G1: property fetch on a template param inside a generic class method must not emit
MixedPropertyFetch — T is an intentionally parameterised type, not truly mixed.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingReturnType
===file===
<?php
/**
 * @template T
 */
class Container {
    /** @param T $item */
    public function __construct(private $item) {}

    public function accessProp(): void {
        // $this->item is T — must not fire MixedPropertyFetch
        $this->item->name;
    }
}
===expect===
