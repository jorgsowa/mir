===description===
Driving a generator with current() and send() is the supported way to use a non-nullable send type.
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
/** @return Generator<int, string, int, void> */
function prompts(): Generator {
    $answer = yield 'How many?';
    echo $answer + 1;
}

$generator = prompts();
echo $generator->current();
$generator->send(3);
