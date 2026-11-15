===description===
A destination that ends in a literal extension or a generated name doesn't let the
client choose the extension.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function upload(string $dir): void {
    move_uploaded_file($_FILES['f']['tmp_name'], $dir . '/' . basename($_FILES['f']['name']) . '.txt');
    move_uploaded_file($_FILES['f']['tmp_name'], $dir . '/' . bin2hex(random_bytes(16)));
    $name = basename($_FILES['f']['name']);
    $name = 'avatar.jpg';
    move_uploaded_file($_FILES['f']['tmp_name'], "$dir/$name");
}
