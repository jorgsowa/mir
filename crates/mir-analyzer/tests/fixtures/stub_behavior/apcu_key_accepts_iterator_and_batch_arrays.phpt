===description===
`apcu_delete()` accepts an `APCUIterator`, and `apcu_store()`/`apcu_add()`
accept an array of key => value pairs with the value argument omitted. The
return type follows the key: a bool for a single key or an iterator, the
failed (or, for `apcu_exists()`, existing) keys for an array.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function clear(string $namespace): bool {
    return apcu_delete(new APCUIterator(sprintf('/^%s/', preg_quote($namespace, '/')), APC_ITER_KEY));
}

function deleteOne(string $id): bool {
    return apcu_delete($id);
}

/** @param list<string> $ids */
function deleteMany(array $ids): void {
    $failed = apcu_delete($ids);
    /** @mir-check $failed is list<string> */
}

function exists(string $id): bool {
    return apcu_exists($id);
}

/** @param list<string> $ids */
function existsMany(array $ids): void {
    $found = apcu_exists($ids);
    /** @mir-check $found is array<string, true> */
}

function storeOne(string $id, mixed $value): bool {
    return apcu_store($id, $value, 60) && apcu_add($id . '.lock', 1);
}

/**
 * @param array<string, mixed> $values
 * @return list<string>
 */
function storeMany(array $values, int $lifetime): array {
    if (false === $failures = apcu_store($values, null, $lifetime)) {
        $failures = $values;
    }
    /** @mir-check $failures is array<string, mixed>|array<string, int> */
    return array_keys($failures);
}

/** @param array<string, mixed> $values */
function addMany(array $values): void {
    $failures = apcu_add($values);
    /** @mir-check $failures is array<string, int>|false */
}

function rejectsOtherKeys(): void {
    apcu_delete(new ArrayIterator([]));
//              ^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $key of apcu_delete() expects 'string|array<int|string, string>|APCUIterator', got 'ArrayIterator'
}
