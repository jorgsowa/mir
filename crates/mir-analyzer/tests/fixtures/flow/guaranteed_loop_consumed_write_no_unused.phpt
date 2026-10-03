===description===
A write made before a do-while is consumed on every valid path through the
loop body. The impossible zero-iteration path must not resurrect it as an
UnusedVariable after the loop merge.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {}

function consume(Box $box): void {}

function test(bool $keepGoing): void {
    $box = new Box;
    do {
        consume($box);
    } while ($keepGoing);
}
===expect===
