===description===
foreach sends null on every step, so a generator with a non-nullable send type is not safe to iterate.
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

foreach (prompts() as $prompt) {
//       ^^^^^^^^^ InvalidGeneratorIteration: foreach sends null to 'Generator<int, string, int, void>', but its send type is 'int'
    echo $prompt;
}
