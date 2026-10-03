===description===
A valid array<K, array{...}> return is not spuriously flagged (TKeyedArray<:TKeyedArray).
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @return array<string, array{id: int, name: string}>
 */
function getItems(): array {
    return ['a' => ['id' => 1, 'name' => 'x']];
}

/**
 * @return list<array{id: int}>
 */
function getListItems(): array {
    return [['id' => 1]];
}
===expect===
