===description===
A private or protected property does not satisfy an object shape; stdClass and classes with __get() are not checked.
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
final class Vault {
    private string $key = 'secret';
    protected int $size = 1;
}

final class Magic {
    public function __get(string $name): mixed {
        return $name;
    }
}

/** @param object{key: string} $holder */
function unlock(object $holder): void {}

unlock(new Vault());
//     ^^^^^^^^^^^ InvalidArgument: Argument $holder of unlock() expects 'object{key: string}', got 'Vault'
unlock(new Magic());
unlock(new stdClass());
unlock((object) ['key' => 'open']);
