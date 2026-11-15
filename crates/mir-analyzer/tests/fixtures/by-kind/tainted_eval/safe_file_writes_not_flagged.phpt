===description===
Request data exported with `var_export()` or encoded with `rawurlencode()` or
`base64_encode()`, which can't produce PHP syntax, written to a file that isn't a PHP
script, or a PHP file written from program data is fine.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function saveConfig(array $settings): void {
    file_put_contents('cache/config.php', '<?php return ' . var_export($_POST['cfg'], true) . ';');
    file_put_contents('cache/data.json', (string) json_encode($_POST['cfg']));
    file_put_contents('uploads/notes.txt', $_POST['notes']);
    file_put_contents('cache/settings.php', '<?php return ' . var_export($settings, true) . ';');
    file_put_contents('cache/blob.php', '<?php return "' . rawurlencode($_POST['blob']) . '";');
    file_put_contents('cache/key.php', "<?php // KEY=" . base64_encode($_POST['key']) . "\nreturn 1;\n");
}
