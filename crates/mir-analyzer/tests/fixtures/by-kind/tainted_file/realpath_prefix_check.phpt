===description===
A `realpath()` result that passes `str_starts_with($p, $dir)` stays inside the trusted directory.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$p = realpath('/d/' . $_GET['x']);
if ($p !== false && str_starts_with($p, '/d/')) {
    readfile($p);
}
$q = '/d/' . $_GET['x'];
if (str_starts_with($q, '/d/')) {
    readfile($q);
//  ^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}
