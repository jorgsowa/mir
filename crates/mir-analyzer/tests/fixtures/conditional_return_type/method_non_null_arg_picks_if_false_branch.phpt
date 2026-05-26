===description===
conditional return type on method resolves to if-false branch when non-null arg is passed
===config===
suppress=UnusedParam
===file===
<?php

/**
 * @template T of object
 */
class Box {}

class Factory {
    /**
     * @template T of object
     * @param class-string<T>|null $cls
     * @return ($cls is null ? Box<object> : Box<T>)
     */
    public function make(?string $cls = null): mixed { throw new \RuntimeException(); }
}

class Item {}

$factory = new Factory();
$b = $factory->make(Item::class);
/** @mir-check $b is Box<Item> */
echo "ok";
===expect===
