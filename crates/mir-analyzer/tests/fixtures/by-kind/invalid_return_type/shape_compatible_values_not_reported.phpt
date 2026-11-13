===description===
Shape values that fit the declared ones are accepted: int for float, null for nullable, nested lists and shapes, an omitted optional key, and a partial or empty shape.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array{ratio: float, parent: ?int, ids: list<int>, meta: array{depth: int}} */
function full(): array {
    return ['ratio' => 1, 'parent' => null, 'ids' => [1, 2], 'meta' => ['depth' => 3]];
}

/** @return array{id: int, name?: string} */
function withoutOptional(): array {
    return ['id' => 1];
}

/** @return array{id: int, name: string} */
function partial(): array {
    return ['id' => 1];
}

/** @return array{id: int, name: string} */
function none(): array {
    return [];
}
