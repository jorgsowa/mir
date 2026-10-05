===description===
A bare array shape property is accepted where an array-intersection param declares that key as an array type; wrong-kind and wrong-shape values still error.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MismatchingDocblockParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, mixed>&array{actor: array{id: int}&array<string, mixed>, target?: array<string, mixed>} $c */
function takesContext(array $c): void {}

/** @param array<string, mixed>&array{items: list<int>} $c */
function takesList(array $c): void {}

function ok(array $actor, array $target): void {
    takesContext(['actor' => $actor, 'target' => $target]);
    takesContext(['actor' => $actor]);
    takesList(['items' => $target]);
}

function bad(int $i): void {
    takesContext(['actor' => $i]);
    takesContext(['actor' => ['id' => 'x']]);
}
===expect===
InvalidArgument@15:17-15:32: Argument $c of takesContext() expects 'array<string, mixed>&array{'actor': array{'id': int}&array<string, mixed>, 'target'?: array<string, mixed>}', got 'array{'actor': int}'
InvalidArgument@16:17-16:43: Argument $c of takesContext() expects 'array<string, mixed>&array{'actor': array{'id': int}&array<string, mixed>, 'target'?: array<string, mixed>}', got 'array{'actor': array{'id': "x"}}'
