===description===
An anchored pattern on the client's name, or a helper of the program's own given
the upload, counts as an extension check.
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
function byPattern(string $dir): void {
    $name = basename($_FILES['f']['name']);
    if (!preg_match('/\.(png|jpe?g)$/i', $name)) {
        return;
    }
    move_uploaded_file($_FILES['f']['tmp_name'], "$dir/$name");
}

/** @param array<string, mixed> $file */
function isAllowedUpload(array $file): bool {
    return $file['size'] < 1000;
}

function byHelper(string $dir): void {
    if (!isAllowedUpload($_FILES['f'])) {
        return;
    }
    $name = basename($_FILES['f']['name']);
    move_uploaded_file($_FILES['f']['tmp_name'], "$dir/$name");
}
