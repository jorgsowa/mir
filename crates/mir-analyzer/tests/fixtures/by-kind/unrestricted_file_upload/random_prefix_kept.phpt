===description===
A random prefix makes the name unique but still ends in the client's extension.
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
function upload(string $dir): void {
    $name = uniqid() . $_FILES['f']['name'];
    move_uploaded_file($_FILES['f']['tmp_name'], "$dir/$name");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnrestrictedFileUpload: Uploaded file saved under the client's file name with no extension check — possible web shell upload
}
