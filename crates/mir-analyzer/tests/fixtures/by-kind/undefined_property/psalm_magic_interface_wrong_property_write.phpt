===description===
Psalm magic interface wrong property write
===file===
<?php
/**
 * @psalm-property-write string $foo
 * @psalm-seal-properties
 */
interface GetterSetter {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value) : void;
}

/** @psalm-suppress NoInterfaceProperties */
function getFoo(GetterSetter $o) : void {
    $o->bar = "hello";
}
===expect===
UnusedSuppress@14:0-14:0: Suppress annotation for 'NoInterfaceProperties' is never used
NoInterfaceProperties@15:4-15:21: Property $bar is not defined on sealed interface
