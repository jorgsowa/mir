===description===
`resolve_prop_current_type` matched `TNamedObject`/`TSelf`/`TStaticObject`
receivers when looking up a property's declared type, but not `TParent`
— every other object-narrowing function in narrowing.rs treats the four
as equivalent receiver atoms. A `parent`-typed receiver (from a `@return
parent` docblock) silently resolved every property to `mixed`, so
`isset()`/property narrowing on it was inert.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Base {
    public ?string $name = null;
}

class Child extends Base {
    /** @return parent */
    public function getBase() {
        return new Base();
    }

    public function useIsset(): void {
        $b = $this->getBase();
        if (isset($b->name)) {
            /** @mir-check $b->name is string */
            $_ = 1;
        }
    }
}
===expect===
