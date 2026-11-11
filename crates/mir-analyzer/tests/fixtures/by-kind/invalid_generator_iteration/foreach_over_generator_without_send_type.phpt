===description===
A generator with no declared send type, or an inferred one, is iterated freely.
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
/** @return Generator<int, string> */
function twoParams(): Generator { yield 'a'; }
function bare(): Generator { yield 'a'; }
function inferred() { yield 1; yield 2; }
/** @return iterable<int, string> */
function plain(): iterable { return ['a']; }

foreach (twoParams() as $v) { echo $v; }
foreach (bare() as $v) { echo $v; }
foreach (inferred() as $v) { echo $v; }
foreach (plain() as $v) { echo $v; }
