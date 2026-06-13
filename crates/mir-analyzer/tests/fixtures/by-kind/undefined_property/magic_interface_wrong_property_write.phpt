===description===
Magic interface wrong property write
===file===
<?php
/**
 * @property-write string $foo
 * @seal-properties
 */
interface GetterSetter {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value) : void;
}

/** @suppress NoInterfaceProperties */
function getFoo(GetterSetter $o) : void {
    $o->bar = "hello";
}
===expect===
UnusedPsalmSuppress@14:0-14:0: Suppress annotation for 'NoInterfaceProperties' is never used
NoInterfaceProperties@15:5-15:22: Property $bar is not defined on sealed interface
