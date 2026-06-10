===description===
Psalm magic interface property wrong property
===ignore===
TODO
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
