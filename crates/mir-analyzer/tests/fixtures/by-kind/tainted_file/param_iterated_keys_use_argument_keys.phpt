===description===
A parameter whose keys reach a sink is tainted by the argument's keys, not its values.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sinkKeys(array $a): void {
    foreach ($a as $k => $v) {
        file_put_contents('/tmp/' . $k, 'x');
    }
}
function f(): void {
    sinkKeys(['a' => $_GET['x']]);
    sinkKeys([$_GET['x'] => 'a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    $copy = [];
    foreach ($_GET as $v) {
        $copy['fixed'] = $v;
    }
    sinkKeys($copy);
}
