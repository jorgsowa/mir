===description===
A comment mentioning a check, or a string function on something other than the
upload, doesn't check the extension.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <TaintedFile errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
// TODO: validate the extension with pathinfo() or finfo later.
function upload(string $dir): void {
    $name = basename($_FILES['f']['name']);
    move_uploaded_file($_FILES['f']['tmp_name'], "$dir/$name");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnrestrictedFileUpload: Uploaded file saved under the client's file name with no extension check — possible web shell upload
}

function prefixed(string $dir, string $user): void {
    $prefix = substr($user, 0, 2);
    $name = basename($_FILES['f']['name']);
    move_uploaded_file($_FILES['f']['tmp_name'], "$dir/$prefix$name");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnrestrictedFileUpload: Uploaded file saved under the client's file name with no extension check — possible web shell upload
}
