===description===
A file that checks the upload's extension isn't reported, even when the check is
on a separate variable.
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
function upload(string $dir): void {
    $name = basename((string) $_FILES['f']['name']);
    $ext = strtolower(pathinfo($name, PATHINFO_EXTENSION));
    if ($ext !== 'jpg' && $ext !== 'png') {
        return;
    }
    move_uploaded_file($_FILES['f']['tmp_name'], $dir . '/' . $name);
}
