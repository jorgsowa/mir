===description===
The send type is checked for a generator held in a variable or returned by a method, with or without a key.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Session
{
    /** @return Generator<string, int, string, bool> */
    public function steps(): Generator {
        $reply = yield 'first' => 1;
        return $reply === 'ok';
    }
}

/** @param Generator<int, int, non-empty-string, void> $fed */
function run(Session $session, Generator $fed): void {
    foreach ($session->steps() as $name => $step) {
//           ^^^^^^^^^^^^^^^^^ InvalidGeneratorIteration: foreach sends null to 'Generator<string, int, string, bool>', but its send type is 'string'
        echo $name, $step;
    }
    foreach ($fed as $value) {
//           ^^^^ InvalidGeneratorIteration: foreach sends null to 'Generator<int, int, non-empty-string, void>', but its send type is 'non-empty-string'
        echo $value;
    }
}
