===description===
Magic interface wrong property write
===ignore===
TODO
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
