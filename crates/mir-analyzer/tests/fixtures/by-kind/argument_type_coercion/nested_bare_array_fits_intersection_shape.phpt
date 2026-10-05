===description===
A bare array nested in an array literal argument is accepted where the shape property is an array intersection, like at the top level.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{actor: array{id: int}&array<string, mixed>} $c */
function takesShape(array $c): void {}

/** @param array{actor: array{id: int}&array<string, mixed>, meta?: array<string, mixed>} $c */
function takesTwo(array $c): void {}

/** @param list<array{id: int}&array<string, mixed>> $c */
function takesList(array $c): void {}

function ok(array $actor, array $meta): void {
    takesShape($actor);
    takesShape(['actor' => $actor]);
    takesTwo(['actor' => $actor, 'meta' => $meta]);
    takesList([$actor]);
}

function bad(int $i, string $s): void {
    takesShape(['actor' => $i]);
    takesShape(['actor' => $s]);
    takesShape(['other' => []]);
}
===expect===
InvalidArgument@19:15-19:30: Argument $c of takesShape() expects 'array{'actor': array{'id': int}&array<string, mixed>}', got 'array{'actor': int}'
InvalidArgument@20:15-20:30: Argument $c of takesShape() expects 'array{'actor': array{'id': int}&array<string, mixed>}', got 'array{'actor': string}'
InvalidArgument@21:15-21:30: Argument $c of takesShape() expects 'array{'actor': array{'id': int}&array<string, mixed>}', got 'array{'other': array{}}'
