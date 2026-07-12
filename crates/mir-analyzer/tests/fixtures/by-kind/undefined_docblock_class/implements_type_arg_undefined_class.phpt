===description===
UndefinedDocblockClass fires when a class name inside an `@implements`
generic type-argument list does not exist.
===config===
suppress=UnusedParam,MissingReturnType,MissingParamType
===file===
<?php
/**
 * @template TKey
 * @template TValue
 */
interface Collection {
    public function get($key);
}

/** @implements Collection<int, NonExistentValueType> */
class IntCollection implements Collection {
    public function get($key) {
        return null;
    }
}
===expect===
UndefinedDocblockClass@10:0-10:56: Docblock type 'NonExistentValueType' does not exist
