===description===
A `@return` listing only scalars on a function or method with a native class
return type contradicts the hint, so the native type is used at call sites and
the docblock is `MismatchingDocblockReturnType`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Query {
    public function execute(): string { return 'ok'; }
}

/** @return bool */
function load(): Query {
//       ^^^^ MismatchingDocblockReturnType: Docblock return type 'bool' does not match inferred 'Query'
    return new Query();
}

final class Repo {
    /** @return string|null */
    public function find(): Query {
        return new Query();
    }
}

function takesBool(bool $value): void {}

function run(Repo $r): void {
    echo load()->execute();
    echo $r->find()->execute();
    takesBool(load());
//            ^^^^^^ InvalidArgument: Argument $value of takesBool() expects 'bool', got 'Query'
}
