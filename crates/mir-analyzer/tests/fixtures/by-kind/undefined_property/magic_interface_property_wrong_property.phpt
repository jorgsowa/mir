===description===
Magic interface property wrong property
===ignore===
TODO
===file===
<?php
/**
 * @property-read string $foo
 * @seal-properties
 */
interface GetterSetter {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value) : void;
}

/** @suppress NoInterfaceProperties */
function getBar(GetterSetter $o) : string {
    return $o->bar;
}
===expect===
