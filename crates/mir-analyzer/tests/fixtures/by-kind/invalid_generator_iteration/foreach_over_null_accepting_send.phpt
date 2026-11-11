===description===
A send type of void, mixed, null or a nullable type accepts the null that foreach sends.
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
/** @return Generator<int, string, void, void> */
function voidSend(): Generator { yield 'a'; }
/** @return Generator<int, string, mixed, void> */
function mixedSend(): Generator { yield 'a'; }
/** @return Generator<int, string, null, void> */
function nullSend(): Generator { yield 'a'; }
/** @return Generator<int, string, int|null, void> */
function nullableSend(): Generator { yield 'a'; }

foreach (voidSend() as $v) { echo $v; }
foreach (mixedSend() as $v) { echo $v; }
foreach (nullSend() as $v) { echo $v; }
foreach (nullableSend() as $v) { echo $v; }
