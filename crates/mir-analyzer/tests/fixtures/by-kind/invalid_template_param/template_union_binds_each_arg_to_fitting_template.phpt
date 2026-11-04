===description===
For a `T|U` param, an argument binds the template whose bound accepts it, not every template in the union
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Id: string {
    case Null = 'null';
    case Int = 'int';
}

/** @template T */
final class Builtin {}

final class Factory {
    /**
     * @template T of Id
     * @template U of value-of<T>
     * @param T|U $id
     * @return Builtin<T>
     */
    public static function builtin(Id|string $id): Builtin {
        return new Builtin();
    }
}

function f(): void {
    $fromCase = Factory::builtin(Id::Int);
    /** @mir-check $fromCase is Builtin<Id::Int> */
    $fromValue = Factory::builtin('null');
    echo 1;
}
