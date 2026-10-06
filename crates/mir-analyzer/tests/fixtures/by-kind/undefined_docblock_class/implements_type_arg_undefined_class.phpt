===description===
UndefinedDocblockClass fires when a class name inside an `@implements`
generic type-argument list does not exist.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentValueType' does not exist
class IntCollection implements Collection {
    public function get($key) {
        return null;
    }
}
