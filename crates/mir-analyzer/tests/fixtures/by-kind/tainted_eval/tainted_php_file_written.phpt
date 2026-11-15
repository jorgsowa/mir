===description===
Request data written into a file whose name ends in a PHP extension runs as code
when that file is requested or included.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function saveConfig(): void {
    file_put_contents('cache/config.php', '<?php return ' . $_POST['cfg'] . ';');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted code — possible code injection
}

function saveTemplate(string $dir): void {
    $name = basename((string) $_GET['name']);
    file_put_contents("$dir/$name.phtml", $_POST['body']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted code — possible code injection
}
