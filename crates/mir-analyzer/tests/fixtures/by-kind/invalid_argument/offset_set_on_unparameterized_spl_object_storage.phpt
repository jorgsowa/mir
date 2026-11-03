===description===
Writing through `[]` to an SplObjectStorage created without type arguments leaves its value template unbound, so any value is accepted.
===file===
<?php
function track(\stdClass $key, string $id): int {
    $responses = new \SplObjectStorage();
    $responses[$key] = $id;
    $responses[$key] = ['id' => $id];
    return count($responses);
}

/** @param \SplObjectStorage<\stdClass, string> $names */
function name(\SplObjectStorage $names, \stdClass $key): void {
    $names[$key] = 'ok';
}
