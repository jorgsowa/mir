===description===
Psalm magic interface property wrong property
===config===
suppress=MixedReturnStatement
===file===
<?php
/**
 * @psalm-property-read string $foo
 * @psalm-seal-properties
 */
interface GetterSetter {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value) : void;
}

/** @psalm-suppress NoInterfaceProperties */
function getBar(GetterSetter $o) : string {
    return $o->bar;
}
===expect===
UnusedPsalmSuppress@14:0-14:0: Suppress annotation for 'NoInterfaceProperties' is never used
NoInterfaceProperties@15:16-15:19: Property $bar is not defined on sealed interface
