===description===
The destination ends in the file name the client sent and nothing checks the
extension, so a `.php` upload lands where the server runs it.
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
function upload(): void {
    $target = 'uploads/';
    $target .= basename($_FILES['uploaded']['name']);
    move_uploaded_file($_FILES['uploaded']['tmp_name'], $target);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnrestrictedFileUpload: Uploaded file saved under the client's file name with no extension check — possible web shell upload
}

function trustsContentType(string $dir): void {
    if ($_FILES['avatar']['type'] === 'image/jpeg') {
        $name = basename($_FILES['avatar']['name']);
        move_uploaded_file($_FILES['avatar']['tmp_name'], "$dir/$name");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnrestrictedFileUpload: Uploaded file saved under the client's file name with no extension check — possible web shell upload
    }
}

function lowercased(string $dir): void {
    $name = strtolower(basename($_FILES['doc']['name']));
    move_uploaded_file($_FILES['doc']['tmp_name'], $dir . '/' . time() . '_' . $name);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnrestrictedFileUpload: Uploaded file saved under the client's file name with no extension check — possible web shell upload
}
