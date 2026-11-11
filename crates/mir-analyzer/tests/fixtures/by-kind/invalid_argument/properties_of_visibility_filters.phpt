===description===
The `public-`, `protected-` and `private-` forms keep only the properties of that visibility.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Session {
    public string $id = '';
    protected int $ttl = 0;
    private bool $dirty = false;
}

/** @param public-properties-of<Session> $data */
function exportPublic(array $data): void {}

/** @param protected-properties-of<Session> $data */
function exportProtected(array $data): void {}

/** @param private-properties-of<Session> $data */
function exportPrivate(array $data): void {}

exportPublic(['id' => 'abc']);
exportPublic(['id' => 12]);
//           ^^^^^^^^^^^^ InvalidArgument: Argument $data of exportPublic() expects 'array{'id': string}', got 'array{'id': 12}'
exportProtected(['ttl' => 60]);
exportProtected(['ttl' => '60s']);
//              ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $data of exportProtected() expects 'array{'ttl': int}', got 'array{'ttl': "60s"}'
exportPrivate(['dirty' => true]);
exportPrivate(['dirty' => 'yes']);
//            ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $data of exportPrivate() expects 'array{'dirty': bool}', got 'array{'dirty': "yes"}'
