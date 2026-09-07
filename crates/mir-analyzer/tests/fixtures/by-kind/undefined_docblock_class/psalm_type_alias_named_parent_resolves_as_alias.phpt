===description===
FP-L11: `parse_type_string` resolves the bare word `parent` to the `parent`
keyword sentinel before any alias table exists to consult, so a
`@psalm-type Parent = SomeClass` alias used at a `@return Parent` site
never reached the alias-expansion pass's `TNamedObject` arm — it stayed the
keyword and got compared against the enclosing class's real parent,
flagging InvalidReturnType even though `Test` has no parent at all.
===file===
<?php
class SomeClass {}

class Test {
    /**
     * @psalm-type Parent = SomeClass
     * @return Parent
     */
    public function make(): SomeClass {
        return new SomeClass();
    }
}
===expect===
