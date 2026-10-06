===description===
@readonly on a static property was never enforced at all -- the
StaticPropertyAccess write arm in assignment.rs never read
prop_def.is_readonly, unlike the instance-property arm a few hundred
lines above it.
===file===
<?php
class Registry {
    /** @readonly */
    public static array $items = [];
}

Registry::$items = ['x'];
//<^^^^^^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Registry::$items outside of constructor
